use crate::domain::{Task, TaskRepository as TaskRepositoryTrait};
use crate::infrastructure::llm::GeminiClient;
use crate::infrastructure::preprocessor::TranscriptPreprocessor;
use crate::infrastructure::tasks::TaskRepository;
use crate::infrastructure::transcription::Transcriber;
use crate::use_cases::transcode::TranscodeUseCase;
use anyhow::Result;
use std::path::Path;
use tracing::{info, warn};

pub struct ProcessUseCase {
    gemini: GeminiClient,
}

impl ProcessUseCase {
    pub fn new(gemini: GeminiClient) -> Self {
        Self { gemini }
    }

    pub async fn execute_session(&self, task: Task) -> Result<()> {
        let transcriber = Transcriber::new(self.gemini.clone());
        let preprocessor = TranscriptPreprocessor::new();
        let transcoder = TranscodeUseCase::new();

        for file_path in task.file_paths {
            info!("Transcribing {} (via Gemini)...", file_path);
            let transcript = transcriber.transcribe(&file_path).await?;

            info!("Preprocessing transcript (Rust)...");
            let cleaned = preprocessor.process(&transcript)?;

            info!("Summarizing transcript...");
            let prompt = format!("以下の会話ログを要約してください。重要なトピックや出来事を箇条書きで抽出してください。特定のプラットフォームに依存しない表現を心がけてください。\n\n{}", cleaned);
            let summary = self.gemini.generate_content(&prompt).await?;

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

            let summary_out_path = format!("data/summaries/{}_summary.txt", date_str);
            crate::infrastructure::fs_utils::atomic_write(&summary_out_path, summary)?;
            info!("Summary saved to {}", summary_out_path);

            let is_lossless_or_raw = Path::new(&file_path)
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "wav" | "flac"))
                .unwrap_or(false);

            if is_lossless_or_raw {
                match transcoder.execute(&file_path).await {
                    Ok(opus_path) => info!("Archived recording as {}", opus_path),
                    Err(e) => warn!(
                        "Transcoding failed for {} (keeping original file): {}",
                        file_path, e
                    ),
                }
            }
        }

        let repo = TaskRepository::new("data/tasks.json");
        repo.update_status(&task.id, "completed")?;
        info!("Task {} marked as completed", task.id);
        Ok(())
    }
}
