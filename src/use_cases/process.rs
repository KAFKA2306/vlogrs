use crate::infrastructure::api::GeminiClient;
use crate::infrastructure::tasks::Task;
use log::info;
use std::process::Command;

pub struct ProcessUseCase {
    gemini: GeminiClient,
}

impl ProcessUseCase {
    pub fn new(gemini: GeminiClient) -> Self {
        Self { gemini }
    }

    pub async fn execute_session(&self, task: Task) -> anyhow::Result<()> {
        info!("Executing process_session for task {}", task.id);

        for file_path in task.file_paths {
            info!("Transcribing {} (via Python)...", file_path);
            let status = Command::new("uv")
                .args([
                    "run",
                    "python",
                    "-m",
                    "src.cli",
                    "transcribe",
                    "--file",
                    &file_path,
                ])
                .status()?;

            if !status.success() {
                anyhow::bail!("Transcription failed for {}", file_path);
            }

            let path = std::path::Path::new(&file_path);
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let transcript_path = format!("data/transcripts/{}.txt", stem);
            let transcript = std::fs::read_to_string(&transcript_path)?;

            info!("Summarizing transcript...");
            let prompt = format!("以下のVRChatでの会話ログを要約してください。重要なトピックや出来事を箇条書きで抽出してください。\n\n{}", transcript);
            let summary = self.gemini.generate_content(&prompt).await?;

            let date_str = stem.split('_').next().unwrap_or("unknown");
            let summary_out_path = format!("data/summaries/{}_summary.txt", date_str);
            std::fs::write(&summary_out_path, summary)?;
            info!("Summary saved to {}", summary_out_path);
        }

        Ok(())
    }
}
