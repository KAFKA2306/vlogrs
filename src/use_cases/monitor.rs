use crate::domain::{AudioRecorder, Environment, ProcessMonitor, TaskRepository as TaskRepositoryTrait};
use crate::infrastructure::llm::GeminiClient;
use crate::use_cases::process::ProcessUseCase;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tokio::time::sleep;
use tracing::{error, info, warn};

pub struct MonitorUseCase {
    audio_recorder: Arc<dyn AudioRecorder>,
    process_monitor: Arc<tokio::sync::Mutex<dyn ProcessMonitor>>,
    task_repository: Arc<dyn TaskRepositoryTrait>,
    environment: Arc<dyn Environment>,
    gemini_client: GeminiClient,
    check_interval: u64,
    audio_device: Option<String>,
    silence_threshold: f32,
}

impl MonitorUseCase {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        audio_recorder: Arc<dyn AudioRecorder>,
        process_monitor: Arc<tokio::sync::Mutex<dyn ProcessMonitor>>,
        task_repository: Arc<dyn TaskRepositoryTrait>,
        environment: Arc<dyn Environment>,
        gemini_client: GeminiClient,
        check_interval: u64,
        audio_device: Option<String>,
        silence_threshold: f32,
    ) -> Self {
        Self {
            audio_recorder,
            process_monitor,
            task_repository,
            environment,
            gemini_client,
            check_interval,
            audio_device,
            silence_threshold,
        }
    }

    pub async fn execute(&self) -> Result<()> {
        self.environment.ensure_directories()?;

        let watcher = crate::infrastructure::watcher::FileWatcher::new("data/cloud_sync");
        watcher.start()?;

        let gemini = self.gemini_client.clone();
        let repo = self.task_repository.clone();
        tokio::spawn(async move {
            let use_case = ProcessUseCase::new(gemini);
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
                        if let Err(e) = use_case.execute_session(task).await {
                            error!("Task execution failed: {}", e);
                        } else {
                            info!("Task completed");
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
        loop {
            let running = self.process_monitor.lock().await.is_running();
            if running && !is_recording {
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                let path = std::path::PathBuf::from(format!("data/recordings/{}.wav", timestamp));
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
                }
            } else if !running && is_recording {
                if let Ok(Some(path)) = self.audio_recorder.stop() {
                    info!("Session recording saved to: {:?}", path);
                    if let Err(e) = self
                        .task_repository
                        .add("process_session", vec![path.to_string_lossy().to_string()])
                    {
                        error!("Failed to add task: {}", e);
                    }
                }
                is_recording = false;
            }
            sleep(Duration::from_secs(self.check_interval)).await;
        }
    }
}
