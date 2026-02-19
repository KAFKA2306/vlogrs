use crate::domain::{EventRepository, LifeEvent, SourceType};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use tracing::{error, info};

pub struct ActivitySyncUseCase {
    repo: Arc<dyn EventRepository>,
}

impl ActivitySyncUseCase {
    pub fn new(repo: Arc<dyn EventRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, file_path: &str) {
        info!("Syncing activity log: {}", file_path);
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            if line.trim().is_empty() {
                continue;
            }

            let json: serde_json::Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to parse JSONL line: {}", e);
                    continue;
                }
            };

            // Map Windows Activity JSON to LifeEvent
            // Expected format (from windows_logger.md):
            // { "timestamp": "...", "type": "WindowFocus", "data": { ... } }

            let source_type = match json["type"].as_str() {
                Some("WindowFocus") => SourceType::WindowsActivity,
                Some("MediaPlaying") => SourceType::WindowsActivity,
                _ => SourceType::WindowsActivity,
            };

            let timestamp = match json["timestamp"].as_str() {
                Some(ts) => chrono::DateTime::parse_from_rfc3339(ts)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                None => chrono::Utc::now(),
            };

            let payload = json["data"].clone();

            let event = LifeEvent {
                id: uuid::Uuid::now_v7(),
                timestamp,
                source: source_type,
                payload,
            };

            self.repo.save(&event).await;
        }

        info!("Sync completed for {}", file_path);
        // Optional: archive file after processing
        let archive_path = format!("{}.processed", file_path);
        // Optional: archive file after processing
        let _archive_path = format!("{}.processed", file_path);
        std::fs::rename(file_path, _archive_path).unwrap();
    }
}
