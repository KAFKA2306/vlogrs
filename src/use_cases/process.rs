use crate::infrastructure::llm::GeminiClient;
use crate::infrastructure::preprocessor::TranscriptPreprocessor;
use crate::infrastructure::tasks::{Task, TaskRepository};
use crate::infrastructure::transcription::Transcriber;
use log::info;
use std::path::Path;

pub struct ProcessUseCase {
    gemini: GeminiClient,
}

impl ProcessUseCase {
    pub fn new(gemini: GeminiClient) -> Self {
        Self { gemini }
    }

    pub async fn execute_session(&self, task: Task) {
        let transcriber = Transcriber::new(self.gemini.clone());
        let preprocessor = TranscriptPreprocessor::new();

        for file_path in task.file_paths {
            info!("Transcribing {} (via Gemini)...", file_path);
            let transcript = transcriber.transcribe(&file_path).await;

            info!("Preprocessing transcript (Rust)...");
            let cleaned = preprocessor.process(&transcript);

            info!("Summarizing transcript...");
            let prompt = format!("以下の会話ログを要約してください。重要なトピックや出来事を箇条書きで抽出してください。特定のプラットフォームに依存しない表現を心がけてください。\n\n{}", cleaned);
            let summary = self.gemini.generate_content(&prompt).await;

            let path = Path::new(&file_path);
            let stem = path.file_stem()
                .expect("Invalid file stem")
                .to_str()
                .expect("Invalid unicode in filename");

            let date_str = stem.split('_').next().expect("Invalid filename format");
            let summary_out_path = format!("data/summaries/{}_summary.txt", date_str);
            crate::infrastructure::fs_utils::atomic_write(&summary_out_path, summary).expect("Failed to write summary");
            info!("Summary saved to {}", summary_out_path);
        }

        let repo = TaskRepository::new("data/tasks.json");
        repo.update_status(&task.id, "completed");
        info!("Task {} marked as completed", task.id);
    }
}
