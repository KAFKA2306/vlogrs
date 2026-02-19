use crate::domain::{ContentGenerator, Task, TaskRepository as TaskRepositoryTrait};
use crate::use_cases::transcode::TranscodeUseCase;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

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

    pub async fn execute_session(&self, task: &Task) -> Result<()> {
        let transcoder = TranscodeUseCase::new();

        for file_path in &task.file_paths {
            info!("Transcribing {} (via Gemini)...", file_path);
            let transcript = self.gemini.transcribe(file_path).await?;

            info!("Preprocessing transcript (Rust)...");
            let preprocessor = crate::infrastructure::preprocessor::TranscriptPreprocessor::new();
            let cleaned = preprocessor.process(&transcript)?;

            // Event Overlay Logic (Milestone 54)
            let path = Path::new(&file_path);
            let stem = path
                .file_stem()
                .ok_or_else(|| anyhow::anyhow!("Invalid file stem"))?
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid unicode in filename"))?;

            let date_str = stem
                .split('_')
                .next()
                .ok_or_else(|| anyhow::anyhow!("Invalid filename format"))?;

            // Attempt to parse timestamp from filename: YYYYMMDD_HHMMSS
            let start_time = chrono::NaiveDateTime::parse_from_str(
                &stem.split('_').take(2).collect::<Vec<_>>().join("_"),
                "%Y%m%d_%H%M%S",
            )
            .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc))
            .unwrap_or(chrono::Utc::now() - chrono::Duration::minutes(30));

            let end_time = start_time + chrono::Duration::minutes(30); // Assume 30 min window for context

            let activities = self
                .event_repository
                .find_by_timerange(start_time, end_time)
                .await?;

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
            let prompt = format!(
                "以下の会話ログと、その間のアクティビティログを元に、このセッションの要約を作成してください。\n\n### アクティビティログ\n{}\n\n### 会話ログ\n{}\n\n要約は箇条書きで、重要なトピックを抽出してください。",
                activity_context, cleaned
            );
            let summary = self.gemini.generate_content(&prompt).await?;

            info!("Verifying summary accuracy (Self-Consistency)...");
            let verify_result = self
                .curator
                .verify_summary(&summary, &cleaned, &activity_context)
                .await;
            info!(
                "Summary Verification: Score={}, Reason={}",
                verify_result.faithfulness_score, verify_result.reasoning
            );

            let summary_out_path = format!("data/summaries/{}_summary.txt", date_str);
            crate::infrastructure::fs_utils::atomic_write(&summary_out_path, summary)?;
            info!("Summary saved to {}", summary_out_path);

            let is_lossless_or_raw = Path::new(&file_path)
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "wav" | "flac"))
                .unwrap_or(false);

            if is_lossless_or_raw {
                match transcoder.execute(file_path).await {
                    Ok(opus_path) => info!("Archived recording as {}", opus_path),
                    Err(e) => warn!(
                        "Transcoding failed for {} (keeping original file): {}",
                        file_path, e
                    ),
                }
            }
        }

        self.task_repository.update_status(&task.id, "completed")?;
        info!("Task {} marked as completed", task.id);
        Ok(())
    }
}
