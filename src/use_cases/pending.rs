use log::info;
use std::fs;
use std::path::{Path, PathBuf};

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
        let summary_dir: &Path = Path::new("data/summaries");
        if !summary_dir.exists() {
            return;
        }

        for entry in fs::read_dir(summary_dir).unwrap() {
            let entry: fs::DirEntry = entry.unwrap();
            let path: PathBuf = entry.path();
            if path.extension().unwrap_or_default() != "txt" {
                continue;
            }

            let file_stem: &str = path.file_stem().unwrap().to_str().unwrap();
            let date: &str = file_stem.split('_').next().unwrap();

            let novel_path: String = format!("data/novels/{}.md", date);
            if !Path::new(&novel_path).exists() {
                info!("Found pending novel for date: {}", date);
                info!("  Run: task novel:build date={}", date);
            }
        }
    }
}
