use crate::infrastructure::api::SupabaseClient;
use crate::infrastructure::settings::Settings;
use anyhow::{Context, Result};
use std::fs;
use tracing::info;

pub struct SyncUseCase {
    settings: Settings,
}

impl SyncUseCase {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub async fn execute(&self) -> Result<()> {
        let client = SupabaseClient::new(
            self.settings.supabase_url.clone(),
            self.settings.supabase_service_role_key.clone(),
        );

        if self.settings.supabase_url.is_empty() {
            tracing::warn!("Supabase URL is not set. Skipping sync.");
            return Ok(());
        }

        let summaries_dir = "data/summaries";
        if !std::path::Path::new(summaries_dir).exists() {
            return Ok(());
        }

        let summaries = fs::read_dir(summaries_dir).context("Failed to read summaries directory")?;

        for entry in summaries {
            let entry = entry.context("Failed to read summary entry")?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                let content = fs::read_to_string(&path).context("Failed to read summary content")?;

                let file_stem = path
                    .file_stem()
                    .ok_or_else(|| anyhow::anyhow!("Invalid file stem"))?
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid unicode in filename"))?;

                let date_str = file_stem
                    .split('_')
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Invalid summary filename format"))?;

                let data = serde_json::json!({
                    "file_path": path.to_string_lossy(),
                    "date": date_str,
                    "content": content,
                    "tags": ["summary"]
                });

                client.upsert("daily_entries", &data).await?;
                info!("Synced {}", path.display());
            }
        }
        Ok(())
    }
}
