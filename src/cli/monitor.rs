use crate::infrastructure;
use crate::infrastructure::settings::Settings;
use crate::use_cases;
use std::sync::Arc;
use tracing::info;
pub async fn run(spawn_worker: bool) {
    let settings: Settings = if spawn_worker {
        Settings::new().unwrap()
    } else {
        Settings::new_allow_missing_gemini().unwrap()
    };
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
    let (gemini, curator): (
        Arc<dyn crate::domain::ContentGenerator>,
        Arc<dyn crate::domain::Curator>,
    ) = if spawn_worker {
        let client = Arc::new(infrastructure::llm::GeminiClient::new(
            settings.google_api_key.clone(),
            settings.gemini_model.clone(),
            prompts,
        ));
        (client.clone(), client)
    } else {
        let noop = Arc::new(infrastructure::llm::NoopGemini::new());
        (noop.clone(), noop)
    };
    let event_repo = Arc::new(
        infrastructure::db::EventRepository::new(&settings.db_path.to_string_lossy()).await,
    );
    let activity_sync = Arc::new(use_cases::sync_activity::ActivitySyncUseCase::new(
        event_repo.clone(),
    ));
    let recording_dir = if spawn_worker {
        settings.recording_dir.clone()
    } else {
        std::path::PathBuf::from(crate::domain::constants::RECORDINGS_DIR)
    };
    info!(
        "Monitor config: spawn_worker={}, recording_dir={:?}",
        spawn_worker, recording_dir
    );
    let use_case = use_cases::monitor::MonitorUseCase::new(
        recorder,
        monitor,
        repo,
        Arc::new(infrastructure::fs_utils::LocalEnvironment),
        gemini.clone(),
        curator,
        watcher,
        activity_sync,
        event_repo,
        settings.check_interval,
        recording_dir,
        settings.audio_device,
        settings.silence_threshold,
        settings.start_debounce_secs,
        settings.stop_grace_secs,
        settings.min_recording_secs,
    );
    use_case.execute(spawn_worker).await;
}
