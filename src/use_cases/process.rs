use crate::domain::{ContentGenerator, Task};
use crate::use_cases::transcode::TranscodeUseCase;
use std::path::Path;
use std::sync::Arc;
use tracing::info;
pub struct ProcessUseCase {
    gemini: Arc<dyn ContentGenerator>,
    event_repository: Arc<dyn crate::domain::EventRepository>,
    curator: Arc<dyn crate::domain::Curator>,
}
impl ProcessUseCase {
    pub fn new(
        gemini: Arc<dyn ContentGenerator>,
        event_repository: Arc<dyn crate::domain::EventRepository>,
        curator: Arc<dyn crate::domain::Curator>,
    ) -> Self {
        Self {
            gemini,
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
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s,
                None => {
                    tracing::error!("Invalid file stem: {}", file_path);
                    continue;
                }
            };

            let date_str = stem.split('_').next().unwrap_or("unknown");
            let start_time = chrono::NaiveDateTime::parse_from_str(
                &stem.split('_').take(2).collect::<Vec<_>>().join("_"),
                "%Y%m%d_%H%M%S",
            )
            .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc))
            .ok();

            if start_time.is_none() {
                tracing::warn!(
                    "Skipping summary/activity overlay for {} (unsupported format)",
                    file_path
                );
                // Continue with just transcoding if needed, or skip entirely.
                // For now, let's just use dummy time for summary if we really want to process it,
                // but the current logic highly depends on start_time for activity overlay.
            }
            let start_time = start_time.unwrap_or_else(chrono::Utc::now);
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
            let daily_summary = if Path::new(&summary_out_path).exists() {
                let existing = std::fs::read_to_string(&summary_out_path).unwrap();
                format!("{existing}\n\n---\n\n## {stem}\n\n{summary}")
            } else {
                format!("## {stem}\n\n{summary}")
            };
            crate::infrastructure::fs_utils::atomic_write(&summary_out_path, daily_summary);
            info!("Daily summary refreshed at {}", summary_out_path);

            let is_lossless_or_raw = Path::new(&file_path)
                .extension()
                .unwrap()
                .to_str()
                .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "wav" | "flac"))
                .unwrap();
            if is_lossless_or_raw {
                match transcoder.execute(file_path).await {
                    Ok(opus_path) => info!("Archived recording as {}", opus_path),
                    Err(e) => tracing::error!(
                        "Transcoding failed for {} (keeping original file): {}",
                        file_path,
                        e
                    ),
                }
            }
        }
        info!("Session processing finished for task {}", task.id);
    }
}
