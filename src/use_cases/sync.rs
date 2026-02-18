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

    pub async fn execute(&self) {
        let client = SupabaseClient::new(
            self.settings.supabase_url.clone(),
            self.settings.supabase_service_role_key.clone(),
        );
        let summaries = fs::read_dir("data/summaries").expect("Failed to read summaries directory");
        
        for entry in summaries {
            let entry = entry.expect("Failed to read summary entry");
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                let content = fs::read_to_string(&path).expect("Failed to read summary content");
                
                let file_stem = path.file_stem()
                    .expect("Invalid file stem")
                    .to_str()
                    .expect("Invalid unicode in filename");
                    
                let date_str = file_stem.split('_')
                    .next()
                    .expect("Invalid summary filename format");
                    
                let data = serde_json::json!({
                    "file_path": path.to_string_lossy(),
                    "date": date_str,
                    "content": content,
                    "tags": ["summary"]
                });
                
                client.upsert("daily_entries", &data).await;
                info!("Synced {}", path.display());
            }
        }
    }
}
