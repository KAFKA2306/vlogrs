use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::info;

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

    pub async fn execute(&self) -> Result<()> {
        let summary_dir = Path::new("data/summaries");
        if !summary_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(summary_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().unwrap_or_default() != "txt" {
                continue;
            }

            let file_stem = path
                .file_stem()
                .ok_or_else(|| anyhow::anyhow!("Invalid file stem"))?
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid unicode in filename"))?;

            let date = file_stem
                .split('_')
                .next()
                .ok_or_else(|| anyhow::anyhow!("Invalid filename format"))?;

            let novel_path = format!("data/novels/{}.md", date);
            if !Path::new(&novel_path).exists() {
                info!("Found pending novel for date: {}", date);
            }
        }
        Ok(())
    }
}
