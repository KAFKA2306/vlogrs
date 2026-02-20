use crate::domain::{
    AudioRecorder, ContentGenerator, Environment, FileWatcher, ProcessMonitor,
    TaskRepository as TaskRepositoryTrait,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, warn};

pub struct MonitorUseCase {
    audio_recorder: Arc<dyn AudioRecorder>,
    process_monitor: Arc<tokio::sync::Mutex<dyn ProcessMonitor>>,
    task_repository: Arc<dyn TaskRepositoryTrait>,
    environment: Arc<dyn Environment>,
    gemini_client: Arc<dyn ContentGenerator>,
    curator: Arc<dyn crate::domain::Curator>,
    watcher: Arc<dyn FileWatcher>,
    activity_sync: Arc<crate::use_cases::sync_activity::ActivitySyncUseCase>,
    event_repository: Arc<dyn crate::domain::EventRepository>,
    check_interval: u64,
    recording_dir: PathBuf,
    audio_device: Option<String>,
    silence_threshold: f32,
    start_debounce_secs: u64,
    stop_grace_secs: u64,
    min_recording_secs: u64,
}

impl MonitorUseCase {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        audio_recorder: Arc<dyn AudioRecorder>,
        process_monitor: Arc<tokio::sync::Mutex<dyn ProcessMonitor>>,
        task_repository: Arc<dyn TaskRepositoryTrait>,
        environment: Arc<dyn Environment>,
        gemini_client: Arc<dyn ContentGenerator>,
        curator: Arc<dyn crate::domain::Curator>,
        watcher: Arc<dyn FileWatcher>,
        activity_sync: Arc<crate::use_cases::sync_activity::ActivitySyncUseCase>,
        event_repository: Arc<dyn crate::domain::EventRepository>,
        check_interval: u64,
        recording_dir: PathBuf,
        audio_device: Option<String>,
        silence_threshold: f32,
        start_debounce_secs: u64,
        stop_grace_secs: u64,
        min_recording_secs: u64,
    ) -> Self {
        Self {
            audio_recorder,
            process_monitor,
            task_repository,
            environment,
            gemini_client,
            curator,
            watcher,
            activity_sync,
            event_repository,
            check_interval,
            recording_dir,
            audio_device,
            silence_threshold,
            start_debounce_secs,
            stop_grace_secs,
            min_recording_secs,
        }
    }

    pub async fn execute(&self) {
        self.environment.ensure_directories();

        self.watcher.start();

        let task_runner = Arc::new(crate::use_cases::task_runner::TaskRunner::new(
            self.gemini_client.clone(),
            self.task_repository.clone(),
            self.event_repository.clone(),
            self.curator.clone(),
            self.activity_sync.clone(),
        ));

        tokio::spawn(async move { task_runner.run().await });

        tokio::spawn(async move { crate::use_cases::HealthMonitor::run().await });

        let mut is_recording = false;
        let mut recording_started_at: Option<Instant> = None;
        let mut running_since: Option<Instant> = None;
        let mut stopped_since: Option<Instant> = None;

        loop {
            let now = Instant::now();
            let running = self.process_monitor.lock().await.is_running();

            if running {
                stopped_since = None;

                if !is_recording {
                    if running_since.is_none() {
                        running_since = Some(now);
                        info!(
                            "Start trigger pending: waiting {}s debounce",
                            self.start_debounce_secs
                        );
                    }

                    if let Some(since) = running_since {
                        if now.duration_since(since).as_secs() >= self.start_debounce_secs {
                            let timestamp =
                                chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                            let path = self.recording_dir.join(format!("{}.wav", timestamp));
                            if true {
                                self.audio_recorder.start(
                                    path,
                                    crate::domain::constants::DEFAULT_SAMPLE_RATE,
                                    crate::domain::constants::DEFAULT_CHANNELS,
                                    self.audio_device.clone(),
                                    self.silence_threshold,
                                );
                                is_recording = true;
                                recording_started_at = Some(now);
                                running_since = None;
                                info!("Recording started.");
                            }
                        }
                    }
                }
            } else {
                running_since = None;

                if is_recording {
                    if stopped_since.is_none() {
                        stopped_since = Some(now);
                        info!(
                            "Stop trigger pending: waiting {}s grace and {}s min-duration",
                            self.stop_grace_secs, self.min_recording_secs
                        );
                    }

                    let grace_elapsed = stopped_since.is_some_and(|since| {
                        now.duration_since(since).as_secs() >= self.stop_grace_secs
                    });
                    let min_elapsed = recording_started_at.is_some_and(|since| {
                        now.duration_since(since).as_secs() >= self.min_recording_secs
                    });

                    if grace_elapsed && min_elapsed {
                        match self.audio_recorder.stop() {
                            Some(path) => {
                                info!("Session recording saved to: {:?}", path);
                                self.task_repository.add(
                                    "process_session",
                                    vec![path.to_string_lossy().to_string()],
                                );
                            }
                            None => warn!("Recorder stopped, but no output path returned"),
                        }

                        is_recording = false;
                        recording_started_at = None;
                        stopped_since = None;
                    }
                }
            }
            sleep(Duration::from_secs(self.check_interval)).await;
        }
    }
}
