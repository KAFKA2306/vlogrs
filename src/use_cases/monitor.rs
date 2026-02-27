use crate::domain::{
    AudioRecorder, ContentGenerator, Environment, FileWatcher, ProcessMonitor,
    TaskRepository as TaskRepositoryTrait,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
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
    pub async fn execute(&self, spawn_worker: bool) {
        self.environment.ensure_directories();
        self.watcher.start();

        self.recover_partial_recordings().await;

        if spawn_worker {
            let task_runner = Arc::new(crate::use_cases::task_runner::TaskRunner::new(
                self.gemini_client.clone(),
                self.task_repository.clone(),
                self.event_repository.clone(),
                self.curator.clone(),
                self.activity_sync.clone(),
            ));
            tokio::spawn(async move { task_runner.run().await });
        } else {
            info!("Task worker disabled; monitor running in audio-only mode.");
        }
        tokio::spawn(async move { crate::use_cases::HealthMonitor::run().await });
        let mut is_recording = false;
        let mut recording_started_at: Option<Instant> = None;
        let mut running_since: Option<Instant> = None;
        let mut stopped_since: Option<Instant> = None;

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
            #[cfg(unix)]
            {
                let mut sigterm =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                        .unwrap();
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {}
                    _ = sigterm.recv() => {}
                }
            }
            #[cfg(not(unix))]
            {
                let _ = tokio::signal::ctrl_c().await;
            }
            let _ = shutdown_tx.send(()).await;
        });

        info!("Monitor loop started. Press Ctrl+C to gracefully stop.");

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Received shutdown signal. Committing active recording and exiting...");
                    if is_recording {
                        if let Some(path) = self.audio_recorder.stop() {
                            info!("Graceful shutdown saved to: {:?}", path);
                            self.task_repository.add(
                                "process_session",
                                vec![path.to_string_lossy().to_string()],
                            );
                        }
                    }
                    std::process::exit(0);
                }
                _ = tokio::time::sleep(Duration::from_secs(self.check_interval)) => {
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
                                    if cfg!(target_os = "windows") {
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
                                    } else {
                                        info!("Recording skipped (non-Windows host)");
                                        running_since = None;
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
                }
            }
        }
    }

    async fn recover_partial_recordings(&self) {
        info!("Scanning for orphaned partial recordings...");
        if let Ok(entries) = std::fs::read_dir(&self.recording_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                    if file_name.ends_with(".wav.part") || file_name.ends_with(".flac.part") {
                        match std::fs::rename(&path, path.with_extension("")) {
                            Ok(_) => {
                                let recovered = path.with_extension("");
                                info!("Recovered orphaned recording: {:?}", recovered);
                                self.task_repository.add(
                                    "process_session",
                                    vec![recovered.to_string_lossy().to_string()],
                                );
                            }
                            Err(e) => warn!("Failed to recover {:?}: {}", path, e),
                        }
                    }
                }
            }
        }
    }
}
