use crate::infrastructure::api::SupabaseClient;
use crate::infrastructure::settings::Settings;
use log::info;
use std::fs;

pub struct SyncUseCase {
    settings: Settings,
}

impl SyncUseCase {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub async fn execute(&self) -> anyhow::Result<()> {
        let client = SupabaseClient::new(
            self.settings.supabase_url.clone(),
            self.settings.supabase_service_role_key.clone(),
        );
        let summaries = fs::read_dir("data/summaries")?;
        for entry in summaries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                let content = fs::read_to_string(&path)?;
                let date_str = path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .split('_')
                    .next()
                    .unwrap();
                let data = serde_json::json!({
                    "file_path": path.to_string_lossy(),
                    "date": date_str,
                    "content": content,
                    "tags": ["summary"]
                });
                client.upsert("daily_entries", data).await?;
                info!("Synced {}", path.display());
            }
        }
        Ok(())
    }
}
