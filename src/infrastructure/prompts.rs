use serde::Deserialize;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct CuratorPrompts {
    pub evaluate: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NovelizerPrompts {
    pub template: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Prompts {
    pub curator: CuratorPrompts,
    pub novelizer: NovelizerPrompts,
}

impl Prompts {
    pub fn load() -> Self {
        let path = Path::new("data/prompts.yaml");
        if !path.exists() {
             panic!("Prompts file not found at: {:?}", path);
        }
        let file = File::open(path).expect("Failed to open data/prompts.yaml");
        serde_yaml::from_reader(file).expect("Failed to parse data/prompts.yaml")
    }
}
