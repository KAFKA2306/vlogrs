use crate::domain::TaskRepository as TaskRepositoryTrait;
use crate::infrastructure::tasks::TaskRepository;
use anyhow::{Context, Result};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use tracing::{error, info};

pub struct FileWatcher {
    path: PathBuf,
}

impl FileWatcher {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn start(&self) -> Result<()> {
        let path = self.path.clone();
        let (tx, rx) = channel();
        let config = Config::default().with_poll_interval(std::time::Duration::from_secs(2));

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
                    info!("New file detected: {:?}", path);
                    let repo = TaskRepository::new("data/tasks.json");
                    if let Err(e) =
                        repo.add("process_session", vec![path.to_string_lossy().to_string()])
                    {
                        error!("Failed to add task for new file: {}", e);
                    }
                }
            }
        });
        Ok(())
    }
}
