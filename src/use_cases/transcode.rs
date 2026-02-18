use anyhow::{Result, Context};
use std::path::Path;
use std::process::Command;
use tracing::{info, error};

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

        // Milestone 46: ffmpeg to Opus
        let status = Command::new("ffmpeg")
            .args(["-y", "-i", input_path, "-c:a", "libopus", "-b:a", "64k", &output_str])
            .status()
            .context("Failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg failed with status {}", status);
        }

        // Milestone 47: Delete original on success
        if let Err(e) = std::fs::remove_file(input_path) {
            error!("Failed to remove original file {}: {}", input_path, e);
        }

        info!("Transcoding complete: {}", output_str);
        Ok(output_str)
    }
}
