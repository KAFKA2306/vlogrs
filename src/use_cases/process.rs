use crate::domain::{ContentGenerator, Task, TaskRepository as TaskRepositoryTrait};
use crate::use_cases::transcode::TranscodeUseCase;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

pub struct ProcessUseCase {
    gemini: Arc<dyn ContentGenerator>,
    task_repository: Arc<dyn TaskRepositoryTrait>,
    event_repository: Arc<dyn crate::domain::EventRepository>,
    curator: Arc<dyn crate::domain::Curator>,
}

impl ProcessUseCase {
    pub fn new(
        gemini: Arc<dyn ContentGenerator>,
        task_repository: Arc<dyn TaskRepositoryTrait>,
        event_repository: Arc<dyn crate::domain::EventRepository>,
        curator: Arc<dyn crate::domain::Curator>,
    ) -> Self {
        Self {
            gemini,
            task_repository,
            event_repository,
            curator,
        }
    }

    pub async fn execute_session(&self, task: &Task) {
        let transcoder = TranscodeUseCase::new();

        for file_path in &task.file_paths {
            info!("Transcribing {} (via Gemini)...", file_path);
            let transcript = self.gemini.transcribe(file_path).await;

            info!("Preprocessing transcript (Rust)...");
            let preprocessor = crate::infrastructure::preprocessor::TranscriptPreprocessor::new();
            let cleaned = preprocessor.process(&transcript);

            let path = Path::new(&file_path);
            let stem = path.file_stem().unwrap().to_str().unwrap();

            let date_str = stem.split('_').next().unwrap();

            let start_time = chrono::NaiveDateTime::parse_from_str(
                &stem.split('_').take(2).collect::<Vec<_>>().join("_"),
                "%Y%m%d_%H%M%S",
            )
            .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc))
            .unwrap_or(chrono::Utc::now() - chrono::Duration::minutes(30));

            let end_time = start_time + chrono::Duration::minutes(30);

            let activities = self
                .event_repository
                .find_by_timerange(start_time, end_time)
                .await;

            let mut activity_context = String::new();
            for event in activities {
                if let crate::domain::SourceType::WindowsActivity = event.source {
                    activity_context.push_str(&format!(
                        "[{}] {:?}\n",
                        event.timestamp.format("%H:%M:%S"),
                        event.payload
                    ));
                }
            }

            info!("Summarizing transcript with activity overlay...");
            let summary = self
                .curator
                .summarize_session(&cleaned, &activity_context)
                .await;

            info!("Verifying summary accuracy (Self-Consistency)...");
            let verify_result = self
                .curator
                .verify_summary(&summary, &cleaned, &activity_context)
                .await;
            info!(
                "Summary Verification: Score={}, Reason={}",
                verify_result.faithfulness_score, verify_result.reasoning
            );

            let summary_out_path =
                crate::domain::constants::SUMMARY_FILE_TEMPLATE.replace("{}", date_str);
            crate::infrastructure::fs_utils::atomic_write(&summary_out_path, summary);
            info!("Summary saved to {}", summary_out_path);

            let is_lossless_or_raw = Path::new(&file_path)
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "wav" | "flac"))
                .unwrap_or(false);

            if is_lossless_or_raw {
                match transcoder.execute(file_path).await {
                    Ok(opus_path) => info!("Archived recording as {}", opus_path),
                    Err(e) => panic!(
                        "Transcoding failed for {} (keeping original file): {}",
                        file_path, e
                    ),
                }
            }
        }

        self.task_repository.update_status(&task.id, "completed");
        info!("Task {} marked as completed", task.id);
    }
}
