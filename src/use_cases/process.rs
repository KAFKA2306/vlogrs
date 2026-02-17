use crate::infrastructure::api::GeminiClient;
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

    pub async fn execute_session(&self, task: Task) -> anyhow::Result<()> {
        let transcriber = Transcriber::new(self.gemini.clone());
        let preprocessor = TranscriptPreprocessor::new();
        for file_path in task.file_paths {
            info!("Transcribing {} (via Gemini)...", file_path);
            let transcript = transcriber.transcribe(&file_path).await?;

            info!("Preprocessing transcript (Rust)...");
            let cleaned = preprocessor.process(&transcript);

            info!("Summarizing transcript...");
            let prompt = format!("以下のVRChatでの会話ログを要約してください。重要なトピックや出来事を箇条書きで抽出してください。\n\n{}", cleaned);
            let summary = self.gemini.generate_content(&prompt).await?;

            let path = std::path::Path::new(&file_path);
            let stem = path.file_stem().unwrap().to_str().unwrap();

            let date_str = stem.split('_').next().unwrap_or("unknown");
            let summary_out_path = format!("data/summaries/{}_summary.txt", date_str);
            std::fs::write(&summary_out_path, summary)?;
            info!("Summary saved to {}", summary_out_path);
        }

        Ok(())
    }
}
