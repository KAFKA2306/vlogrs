use crate::domain::TaskRepository as TaskRepositoryTrait;
use crate::infrastructure::tasks::TaskRepository;
use anyhow::{Context, Result};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use tracing::{error, info};

pub struct FileWatcher {
    path: PathBuf,
}

impl crate::domain::FileWatcher for FileWatcher {
    fn start(&self) -> Result<()> {
        self.start()
    }
}

impl FileWatcher {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn start(&self) -> Result<()> {
        let path = self.path.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let tx = tx.clone();
        let config = Config::default().with_poll_interval(std::time::Duration::from_secs(
            crate::domain::constants::WATCHER_POLL_INTERVAL_SECS,
        ));

        let mut watcher: RecommendedWatcher =
            Watcher::new(tx, config).context("Failed to create file watcher")?;

        watcher
            .watch(&path, RecursiveMode::Recursive)
            .context("Failed to watch directory")?;

        info!("Started watching directory: {:?}", path);

        std::thread::spawn(move || {
            let _watcher = watcher;

            for res in rx {
                let event = match res {
                    Ok(e) => e,
                    Err(e) => {
                        error!("Watch error: {:?}", e);
                        continue;
                    }
                };
                if !matches!(event.kind, EventKind::Create(_)) {
                    continue;
                }

                for path in event.paths {
                    if !path.is_file() {
                        continue;
                    }
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    match ext.to_lowercase().as_str() {
                        "wav" | "flac" => {
                            info!("New audio file: {:?}", path);
                            let repo = TaskRepository::new(
                                crate::infrastructure::settings::Settings::default_tasks_path(),
                            );
                            if let Err(e) = repo.add(
                                crate::domain::constants::TASK_TYPE_PROCESS_SESSION,
                                vec![path.to_string_lossy().to_string()],
                            ) {
                                error!("Failed to add task for audio: {}", e);
                            }
                        }
                        "jsonl" => {
                            info!("New activity log: {:?}", path);
                            let repo = TaskRepository::new(
                                crate::infrastructure::settings::Settings::default_tasks_path(),
                            );
                            if let Err(e) = repo.add(
                                crate::domain::constants::TASK_TYPE_SYNC_ACTIVITY,
                                vec![path.to_string_lossy().to_string()],
                            ) {
                                error!("Failed to add task for activity log: {}", e);
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
        Ok(())
    }
}
