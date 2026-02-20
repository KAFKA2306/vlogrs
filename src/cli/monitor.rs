use crate::infrastructure;
use crate::infrastructure::settings::Settings;
use crate::use_cases;
use std::sync::Arc;
use tracing::info;
pub async fn run() {
    let settings: Settings = Settings::new().unwrap();
    info!("Starting monitor mode...");
    let recorder = Arc::new(infrastructure::audio::AudioRecorder::new());
    let monitor = Arc::new(tokio::sync::Mutex::new(
        infrastructure::process::ProcessMonitor::new(settings.process_names.clone()),
    ));
    let repo = Arc::new(infrastructure::tasks::TaskRepository::new(
        crate::domain::constants::TASKS_PATH,
    ));
    let watcher = Arc::new(infrastructure::watcher::FileWatcher::new(
        crate::domain::constants::CLOUD_SYNC_DIR,
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
    let use_case = use_cases::monitor::MonitorUseCase::new(
        recorder,
        monitor,
        repo,
        Arc::new(infrastructure::fs_utils::LocalEnvironment),
        gemini.clone(),
        gemini,
        watcher,
        activity_sync,
        event_repo,
        settings.check_interval,
        settings.recording_dir,
        settings.audio_device,
        settings.silence_threshold,
        settings.start_debounce_secs,
        settings.stop_grace_secs,
        settings.min_recording_secs,
    );
    use_case.execute().await;
}
