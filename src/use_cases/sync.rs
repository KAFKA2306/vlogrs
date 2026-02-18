use crate::infrastructure::api::SupabaseClient;
use crate::infrastructure::settings::Settings;
use log::info;
use std::fs;
use std::path::PathBuf;

pub struct SyncUseCase {
    settings: Settings,
}

impl SyncUseCase {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub async fn execute(&self) {
        let client: SupabaseClient = SupabaseClient::new(
            self.settings.supabase_url.clone(),
            self.settings.supabase_service_role_key.clone(),
        );
        let summaries: fs::ReadDir = fs::read_dir("data/summaries").unwrap();
        for entry in summaries {
            let entry: fs::DirEntry = entry.unwrap();
            let path: PathBuf = entry.path();
            if path.extension().and_then(|s: &std::ffi::OsStr| s.to_str()) == Some("txt") {
                let content: String = fs::read_to_string(&path).unwrap();
                let date_str: &str = path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .split('_')
                    .next()
                    .unwrap();
                let data: serde_json::Value = serde_json::json!({
                    "file_path": path.to_string_lossy(),
                    "date": date_str,
                    "content": content,
                    "tags": ["summary"]
                });
                client.upsert("daily_entries", data).await;
                info!("Synced {}", path.display());
            }
        }
    }
}
