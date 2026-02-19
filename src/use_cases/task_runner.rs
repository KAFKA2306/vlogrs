use crate::domain::{ContentGenerator, Curator, TaskRepository};
use crate::use_cases::process::ProcessUseCase;
use crate::use_cases::sync_activity::ActivitySyncUseCase;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use crate::domain::constants::{
    STATUS_PENDING, STATUS_PROCESSING, STATUS_COMPLETED, STATUS_FAILED,
    TASK_TYPE_PROCESS_SESSION, TASK_TYPE_SYNC_ACTIVITY, TASK_LOOP_INTERVAL_SECS,
};

pub struct TaskRunner {
    repository: Arc<dyn TaskRepository>,
    process_use_case: ProcessUseCase,
    activity_sync: Arc<ActivitySyncUseCase>,
}

impl TaskRunner {
    pub fn new(
        gemini: Arc<dyn ContentGenerator>,
        repository: Arc<dyn TaskRepository>,
        event_repo: Arc<dyn crate::domain::EventRepository>,
        curator: Arc<dyn Curator>,
        activity_sync: Arc<ActivitySyncUseCase>,
    ) -> Self {
        let process_use_case = ProcessUseCase::new(gemini, repository.clone(), event_repo, curator);
        Self {
            repository,
            process_use_case,
            activity_sync,
        }
    }

    pub async fn run(&self) -> Result<()> {
        loop {
            let tasks = match self.repository.load() {
                Ok(t) => t,
                Err(e) => {
                    error!("Failed to load tasks: {}", e);
                    sleep(Duration::from_secs(TASK_LOOP_INTERVAL_SECS)).await;
                    continue;
                }
            };

            for task in tasks {
                if task.status == STATUS_PENDING {
                    if let Err(e) = self.repository.update_status(&task.id, STATUS_PROCESSING) {
                        error!("Failed to update task status: {}", e);
                        continue;
                    }

                    info!("Processing task: {} ({})", task.id, task.task_type);

                    let result = match task.task_type.as_str() {
                        TASK_TYPE_PROCESS_SESSION => {
                            self.process_use_case.execute_session(&task).await
                        }
                        TASK_TYPE_SYNC_ACTIVITY => {
                            let mut res = Ok(());
                            for file in &task.file_paths {
                                if let Err(e) = self.activity_sync.execute(file).await {
                                    error!("Activity sync failed for {}: {}", file, e);
                                    res = Err(e);
                                    break;
                                }
                            }
                            res
                        }
                        _ => {
                            warn!("Unknown task type: {}", task.task_type);
                            Ok(())
                        }
                    };

                    match result {
                        Ok(_) => {
                            let _ = self.repository.update_status(&task.id, STATUS_COMPLETED);
                            info!("Task completed: {}", task.id);
                        }
                        Err(e) => {
                            error!("Task failed: {}: {}", task.id, e);
                            let _ = self.repository.update_status(&task.id, STATUS_FAILED);
                        }
                    }
                }
            }
            sleep(Duration::from_secs(TASK_LOOP_INTERVAL_SECS)).await;
        }
    }
}
