use anyhow::Context;
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct CuratorPrompts {
    pub evaluate: String,
    pub session_summary: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NovelizerPrompts {
    pub template: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Prompts {
    pub curator: CuratorPrompts,
    pub novelizer: NovelizerPrompts,
    pub transcription: String,
    pub summary_verification: String,
}

impl Prompts {
    pub fn load() -> anyhow::Result<Self> {
        let path = Path::new("data/prompts.yaml");
        if !path.exists() {
            anyhow::bail!("Prompts file not found at: {:?}", path);
        }
        let file = File::open(path).context("Failed to open data/prompts.yaml")?;
        serde_yaml::from_reader(file).context("Failed to parse data/prompts.yaml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompts_yaml_structure_is_valid() {
        // This test ensures that the Rust struct definition matches the actual data/prompts.yaml file.
        // It acts as a "strict pre-verification" to prevent runtime errors on deployment.
        let result = Prompts::load();
        
        match result {
            Ok(prompts) => {
                // Optional: Check some fields to ensure they are not empty
                assert!(!prompts.curator.evaluate.is_empty(), "curator.evaluate should not be empty");
                assert!(!prompts.curator.session_summary.is_empty(), "curator.session_summary should not be empty");
            },
            Err(e) => {
                panic!("Failed to load specific data/prompts.yaml: {:#?}", e);
            }
        }
    }
}
