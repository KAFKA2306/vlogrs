use crate::infrastructure::llm::GeminiClient;
use crate::infrastructure::preprocessor::TranscriptPreprocessor;
use crate::infrastructure::tasks::Task;
use crate::infrastructure::transcription::Transcriber;
use log::info;

pub struct ProcessUseCase {
    gemini: GeminiClient,
}

impl ProcessUseCase {
    pub fn new(gemini: GeminiClient) -> Self {
        Self { gemini }
    }

    pub async fn execute_session(&self, task: Task) {
        let transcriber: Transcriber = Transcriber::new(self.gemini.clone());
        let preprocessor: TranscriptPreprocessor = TranscriptPreprocessor::new();
        for file_path in task.file_paths {
            info!("Transcribing {} (via Gemini)...", file_path);
            let transcript: String = transcriber.transcribe(&file_path).await;

            info!("Preprocessing transcript (Rust)...");
            let cleaned: String = preprocessor.process(&transcript);

            info!("Summarizing transcript...");
            let prompt: String = format!("以下の会話ログを要約してください。重要なトピックや出来事を箇条書きで抽出してください。特定のプラットフォームに依存しない表現を心がけてください。\n\n{}", cleaned);
            let summary: String = self.gemini.generate_content(&prompt).await;

            let path: &std::path::Path = std::path::Path::new(&file_path);
            let stem: &str = path.file_stem().unwrap().to_str().unwrap();

            let date_str: &str = stem.split('_').next().unwrap();
            let summary_out_path: String = format!("data/summaries/{}_summary.txt", date_str);
            std::fs::write(&summary_out_path, summary).unwrap();
            info!("Summary saved to {}", summary_out_path);
        }

        let repo: crate::infrastructure::tasks::TaskRepository = crate::infrastructure::tasks::TaskRepository::new("data/tasks.json");
        repo.update_status(&task.id, "completed");
        info!("Task {} marked as completed", task.id);
    }
}
