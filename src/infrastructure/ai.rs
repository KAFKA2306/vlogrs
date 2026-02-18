use crate::domain::ImageGenerator;
use std::process::{Command, ExitStatus};

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
        let status: ExitStatus = Command::new("uv")
            .arg("run")
            .arg("src/scripts/image_gen.py")
            .arg("--prompt")
            .arg(prompt)
            .arg("--output")
            .arg(output_path)
            .status()
            .unwrap();

        if !status.success() {
            panic!("Image generation failed");
        }
    }
}
