use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use tracing::{error, info};
pub struct TranscodeUseCase;
impl TranscodeUseCase {
    pub fn new() -> Self {
        Self
    }
    pub async fn execute(&self, input_path: &str) -> Result<String> {
        let input = Path::new(input_path);
        let output = input.with_extension("opus");
        let output_str = output.to_str().context("Invalid output path")?.to_string();
        info!("Transcoding {} to Opus...", input_path);
        let status = Command::new(crate::domain::constants::FFMPEG_CMD)
            .args([
                "-y",
                "-i",
                input_path,
                "-c:a",
                "libopus",
                "-ar",
                &crate::domain::constants::DEFAULT_SAMPLE_RATE.to_string(),
                "-ac",
                &crate::domain::constants::DEFAULT_CHANNELS.to_string(),
                "-application",
                "voip",
                "-vbr",
                "on",
                "-compression_level",
                "10",
                "-b:a",
                "24k",
                &output_str,
            ])
            .status()
            .context("Failed to execute ffmpeg")?;
        if !status.success() {
            anyhow::bail!("ffmpeg failed with status {}", status);
        }
        if let Err(e) = std::fs::remove_file(input_path) {
            error!("Failed to remove original file {}: {}", input_path, e);
        }
        info!("Transcoding complete: {}", output_str);
        Ok(output_str)
    }
}
impl Default for TranscodeUseCase {
    fn default() -> Self {
        Self::new()
    }
}
