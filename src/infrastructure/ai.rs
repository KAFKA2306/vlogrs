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
        let status = Command::new("uv")
            .args(["run", "src/scripts/image_gen.py", "--prompt", prompt, "--output", output_path])
            .status()
            .expect("Failed to execute image generation command");

        if !status.success() {
            panic!("Image generation process failed with status: {}", status);
        }
    }
}
