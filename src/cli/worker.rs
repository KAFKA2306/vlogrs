use crate::domain::Environment;
use crate::infrastructure;
use crate::infrastructure::settings::Settings;
use crate::use_cases;
use std::sync::Arc;
use tracing::info;

pub async fn run() {
    let settings: Settings = Settings::new().unwrap();
    let env = infrastructure::fs_utils::LocalEnvironment;
    env.ensure_directories();

    let repo = Arc::new(infrastructure::tasks::TaskRepository::new(
        crate::domain::constants::TASKS_PATH,
    ));
    let prompts = infrastructure::prompts::Prompts::load().unwrap();
    let gemini = Arc::new(infrastructure::llm::GeminiClient::new(
        settings.google_api_key.clone(),
        settings.gemini_model.clone(),
        prompts,
    ));
    let event_repo = Arc::new(
        infrastructure::db::EventRepository::new(&settings.db_path.to_string_lossy()).await,
    );
    let activity_sync = Arc::new(use_cases::sync_activity::ActivitySyncUseCase::new(
        event_repo.clone(),
    ));

    let task_runner = use_cases::task_runner::TaskRunner::new(
        gemini.clone(),
        repo,
        event_repo,
        gemini.clone(),
        activity_sync,
    );

    info!("Starting worker loop (Gemini processing only)...");
    task_runner.run().await;
}
