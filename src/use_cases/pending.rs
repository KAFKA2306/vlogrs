use log::info;
use std::fs;
use std::path::Path;

pub struct PendingUseCase;

impl Default for PendingUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl PendingUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self) {
        let summary_dir = Path::new("data/summaries");
        if !summary_dir.exists() {
            return;
        }

        for entry in fs::read_dir(summary_dir).expect("Failed to read summary directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.extension().unwrap_or_default() != "txt" {
                continue;
            }

            let file_stem = path.file_stem()
                .expect("Invalid file stem")
                .to_str()
                .expect("Invalid unicode in filename");
                
            let date = file_stem.split('_').next().expect("Invalid filename format");

            let novel_path = format!("data/novels/{}.md", date);
            if !Path::new(&novel_path).exists() {
                info!("Found pending novel for date: {}", date);
                info!("  Run: task novel:build date={}", date);
            }
        }
    }
}
