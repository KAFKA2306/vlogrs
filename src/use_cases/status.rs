use crate::infrastructure::tasks::TaskRepository;
use chrono::{Duration, Utc};
use std::fs;
use std::path::Path;

pub struct StatusUseCase;

impl Default for StatusUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self) {
        let now = Utc::now();
        let since = now - Duration::hours(24);

        let repo = TaskRepository::new("data/tasks.json");
        let tasks = repo.load();

        let pending_count = tasks.iter().filter(|t| t.status == "pending").count();
        let processing_count = tasks.iter().filter(|t| t.status == "processing").count();
        let completed_24h = tasks.iter().filter(|t| t.created_at >= since).count();

        let recordings_24h = self.count_recent_files("data/recordings", since.timestamp());
        let runtime_hours = self.estimate_runtime_hours_from_recordings(since.timestamp());

        println!("=== VLog Status (Last 24h) ===");
        println!("Estimated active hours: {:.2}h", runtime_hours);
        println!("New recordings: {}", recordings_24h);
        println!("Tasks completed/created: {}", completed_24h);
        println!("Pending tasks: {}", pending_count);
        println!("Processing tasks: {}", processing_count);
    }

    fn count_recent_files(&self, dir: &str, since_ts: i64) -> usize {
        let path = Path::new(dir);
        if !path.exists() {
            return 0;
        }

        let mut count = 0;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(modified) = meta.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            let modified_ts = Utc::now().timestamp() - elapsed.as_secs() as i64;
                            if modified_ts >= since_ts {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
        count
    }

    fn estimate_runtime_hours_from_recordings(&self, since_ts: i64) -> f64 {
        let path = Path::new("data/recordings");
        if !path.exists() {
            return 0.0;
        }

        let mut total_bytes: u64 = 0;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(modified) = meta.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            let modified_ts = Utc::now().timestamp() - elapsed.as_secs() as i64;
                            if modified_ts < since_ts {
                                continue;
                            }

                            if meta.is_file() {
                                total_bytes += meta.len();
                            }
                        }
                    }
                }
            }
        }

        // Rough estimate for 16-bit PCM mono 16kHz WAV.
        // This is intentionally approximate and used only for status display.
        let bytes_per_second = 16000.0 * 2.0;
        (total_bytes as f64 / bytes_per_second) / 3600.0
    }
}
