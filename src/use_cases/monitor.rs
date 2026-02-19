use crate::domain::{
    AudioRecorder, ContentGenerator, Environment, FileWatcher, ProcessMonitor,
    TaskRepository as TaskRepositoryTrait,
};
use crate::use_cases::process::ProcessUseCase;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::System;
use tokio::time::sleep;
use tracing::{error, info, warn};

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

    pub async fn execute(&self) -> Result<()> {
        self.environment.ensure_directories()?;

        self.watcher.start()?;

        let gemini = self.gemini_client.clone();
        let curator = self.curator.clone();
        let repo = self.task_repository.clone();
        let event_repo = self.event_repository.clone();
        let sync_use_case = self.activity_sync.clone();
        tokio::spawn(async move {
            let process_use_case = ProcessUseCase::new(gemini, repo.clone(), event_repo, curator);
            loop {
                let tasks = match repo.load() {
                    Ok(t) => t,
                    Err(e) => {
                        error!("Failed to load tasks: {}", e);
                        sleep(Duration::from_secs(30)).await;
                        continue;
                    }
                };
                for task in tasks {
                    if task.status == "pending" {
                        if let Err(e) = repo.update_status(&task.id, "processing") {
                            error!("Failed to update task status: {}", e);
                            continue;
                        }
                        match task.task_type.as_str() {
                            "process_session" => {
                                if let Err(e) = process_use_case.execute_session(&task).await {
                                    error!("Session processing failed: {}", e);
                                    let _ = repo.update_status(&task.id, "failed");
                                }
                            }
                            "sync_activity" => {
                                for file in &task.file_paths {
                                    if let Err(e) = sync_use_case.execute(file).await {
                                        error!("Activity sync failed for {}: {}", file, e);
                                        let _ = repo.update_status(&task.id, "failed");
                                    }
                                }
                                let _ = repo.update_status(&task.id, "completed");
                            }
                            _ => warn!("Unknown task type: {}", task.task_type),
                        }
                    }
                }
                sleep(Duration::from_secs(30)).await;
            }
        });

        tokio::spawn(async move {
            let mut sys = System::new_all();
            loop {
                sys.refresh_cpu();
                sys.refresh_memory();
                let cpu = sys.global_cpu_info().cpu_usage();
                let total_mem = sys.total_memory();
                let used_mem = sys.used_memory();
                let mem_pct = if total_mem > 0 {
                    (used_mem as f64 / total_mem as f64) * 100.0
                } else {
                    0.0
                };
                if cpu >= 90.0 || mem_pct >= 90.0 {
                    warn!(
                        "health-check high usage cpu={:.1}% memory={:.1}% - Triggering self-restart",
                        cpu, mem_pct
                    );
                    std::process::exit(1);
                } else {
                    info!("health-check cpu={:.1}% memory={:.1}%", cpu, mem_pct);
                }
                sleep(Duration::from_secs(30)).await;
            }
        });

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
                            if let Err(e) = self.audio_recorder.start(
                                path,
                                16000,
                                1,
                                self.audio_device.clone(),
                                self.silence_threshold,
                            ) {
                                error!("Failed to start recording: {}", e);
                            } else {
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
                            Ok(Some(path)) => {
                                info!("Session recording saved to: {:?}", path);
                                if let Err(e) = self.task_repository.add(
                                    "process_session",
                                    vec![path.to_string_lossy().to_string()],
                                ) {
                                    error!("Failed to add task: {}", e);
                                }
                            }
                            Ok(None) => warn!("Recorder stopped, but no output path returned"),
                            Err(e) => error!("Failed to stop recording: {}", e),
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
