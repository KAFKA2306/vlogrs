use crate::domain::constants::{
    STATUS_COMPLETED, STATUS_PENDING, STATUS_PROCESSING, TASK_LOOP_INTERVAL_SECS,
    TASK_TYPE_PROCESS_SESSION, TASK_TYPE_SYNC_ACTIVITY,
};
use crate::domain::{ContentGenerator, Curator, TaskRepository};
use crate::use_cases::process::ProcessUseCase;
use crate::use_cases::sync_activity::ActivitySyncUseCase;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

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

    pub async fn run(&self) {
        loop {
            let tasks = self.repository.load();

            for task in tasks {
                if task.status == STATUS_PENDING {
                    self.repository.update_status(&task.id, STATUS_PROCESSING);

                    info!("Processing task: {} ({})", task.id, task.task_type);

                    match task.task_type.as_str() {
                        TASK_TYPE_PROCESS_SESSION => {
                            self.process_use_case.execute_session(&task).await;
                            self.repository.update_status(&task.id, STATUS_COMPLETED);
                            info!("Task completed: {}", task.id);
                        }
                        TASK_TYPE_SYNC_ACTIVITY => {
                            for file in &task.file_paths {
                                self.activity_sync.execute(file).await;
                            }
                            self.repository.update_status(&task.id, STATUS_COMPLETED);
                            info!("Task completed: {}", task.id);
                        }
                        _ => {
                            warn!("Unknown task type: {}", task.task_type);
                        }
                    }
                }
            }
            sleep(Duration::from_secs(TASK_LOOP_INTERVAL_SECS)).await;
        }
    }
}
