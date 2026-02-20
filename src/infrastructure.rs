pub mod api;
pub mod audio;
pub mod db;
pub mod fs_utils;
pub mod llm;
pub mod preprocessor;
pub mod process;
pub mod prompts;
pub mod settings;
pub mod tasks;

pub mod watcher;

use crate::domain::ImageGenerator;
use std::process::Command;

pub struct PythonImageGenerator;

impl Default for PythonImageGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonImageGenerator {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl ImageGenerator for PythonImageGenerator {
    async fn generate(&self, prompt: &str, output_path: &str) {
        let status = Command::new(crate::domain::constants::UV_CMD)
            .args([
                "run",
                crate::domain::constants::IMAGE_GEN_SCRIPT,
                "--prompt",
                prompt,
                "--output",
                output_path,
            ])
            .status()
            .expect("Failed to execute image generation command");

        if !status.success() {
            panic!("Image generation process failed with status: {}", status);
        }
    }
}
