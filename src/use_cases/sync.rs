use crate::infrastructure::api::SupabaseClient;
use crate::infrastructure::settings::Settings;
use std::fs;
use tracing::info;
pub struct SyncUseCase {
    settings: Settings,
}
impl SyncUseCase {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }
    pub async fn execute(&self) {
        let client = SupabaseClient::new(
            self.settings.supabase_url.clone(),
            self.settings.supabase_service_role_key.clone(),
        );
        if self.settings.supabase_url.is_empty() {
            tracing::warn!("Supabase URL is not set. Skipping sync.");
            return;
        }
        let summaries_dir = "data/summaries";
        if !std::path::Path::new(summaries_dir).exists() {
            return;
        }
        let summaries = fs::read_dir(summaries_dir).unwrap();
        for entry in summaries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                let content = fs::read_to_string(&path).unwrap();
                let file_stem = path
                    .file_stem()
                    .ok_or_else(|| anyhow::anyhow!("Invalid file stem"))
                    .unwrap()
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid unicode in filename"))
                    .unwrap();
                let date_str = file_stem
                    .split('_')
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Invalid summary filename format"))
                    .unwrap();
                let data = serde_json::json!({
                    "file_path": path.to_string_lossy(),
                    "date": date_str,
                    "content": content,
                    "tags": ["summary"]
                });
                client.upsert("daily_entries", &data).await.unwrap();
                info!("Synced {}", path.display());
            }
        }
    }
}
