use crate::domain::TaskRepository as TaskRepositoryTrait;
use crate::infrastructure::tasks::TaskRepository;
use anyhow::Result;
use chrono::{Duration, Utc};
use std::fs;
use std::path::Path;
use tracing::info;

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

        let repo =
            TaskRepository::new(crate::infrastructure::settings::Settings::default_tasks_path());
        let tasks = repo.load();

        let pending_count = tasks
            .iter()
            .filter(|t| t.status == crate::domain::constants::STATUS_PENDING)
            .count();
        let processing_count = tasks
            .iter()
            .filter(|t| t.status == crate::domain::constants::STATUS_PROCESSING)
            .count();
        let completed_24h = tasks.iter().filter(|t| t.created_at >= since).count();

        let recordings_24h =
            self.count_recent_files(crate::domain::constants::RECORDINGS_DIR, since.timestamp());
        let runtime_hours = self.estimate_runtime_hours_from_recordings(since.timestamp());

        info!("=== VLog Status (Last 24h) ===");
        info!("Estimated active hours: {:.2}h", runtime_hours);
        info!("New recordings: {}", recordings_24h);
        info!("Tasks completed/created: {}", completed_24h);
        info!("Pending tasks: {}", pending_count);
        info!("Processing tasks: {}", processing_count);
    }

    fn count_recent_files(&self, dir: &str, since_ts: i64) -> usize {
        let path = Path::new(dir);
        if !path.exists() {
            return 0;
        }

        fs::read_dir(path)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter(|entry| {
                        entry
                            .metadata()
                            .and_then(|m| m.modified())
                            .map(|t| {
                                t.duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs() as i64
                                    >= since_ts
                            })
                            .unwrap_or(false)
                    })
                    .count()
            })
            .unwrap_or(0)
    }

    fn estimate_runtime_hours_from_recordings(&self, since_ts: i64) -> f64 {
        let path = Path::new(crate::domain::constants::RECORDINGS_DIR);
        if !path.exists() {
            return 0.0;
        }

        let total_bytes: u64 = fs::read_dir(path)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter_map(|entry| {
                        let meta = entry.metadata().ok()?;
                        if !meta.is_file() {
                            return None;
                        }

                        let modified = meta.modified().ok()?;
                        let ts = modified
                            .duration_since(std::time::UNIX_EPOCH)
                            .ok()?
                            .as_secs() as i64;
                        if ts < since_ts {
                            return None;
                        }
                        Some(meta.len())
                    })
                    .sum()
            })
            .unwrap();

        let bytes_per_second = 16000.0 * 2.0;
        (total_bytes as f64 / bytes_per_second) / 3600.0
    }
}
