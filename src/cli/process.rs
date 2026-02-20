use crate::domain;
use crate::infrastructure;
use crate::infrastructure::settings::Settings;
use crate::use_cases;
use std::sync::Arc;
use tracing::info;
pub async fn run(file: String) {
    let settings: Settings = Settings::new().unwrap();
    info!("Processing file: {}", file);
    let prompts = infrastructure::prompts::Prompts::load().unwrap();
    let gemini = Arc::new(infrastructure::llm::GeminiClient::new(
        settings.google_api_key,
        settings.gemini_model,
        prompts,
    ));
    let repo = Arc::new(infrastructure::tasks::TaskRepository::new(
        Settings::default_tasks_path(),
    ));
    let event_repo = Arc::new(
        infrastructure::db::EventRepository::new(&settings.db_path.to_string_lossy()).await,
    );
    let use_case =
        use_cases::process::ProcessUseCase::new(gemini.clone(), repo, event_repo, gemini);
    use_case
        .execute_session(&domain::Task {
            id: "manual".to_string(),
            created_at: chrono::Utc::now(),
            status: crate::domain::constants::STATUS_PROCESSING.to_string(),
            task_type: crate::domain::constants::TASK_TYPE_PROCESS_SESSION.to_string(),
            file_paths: vec![file],
        })
        .await;
}
