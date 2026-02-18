use crate::infrastructure::audio::AudioRecorder;
use crate::infrastructure::llm::GeminiClient;
use crate::infrastructure::process::ProcessMonitor;
use crate::infrastructure::settings::Settings;
use crate::infrastructure::tasks::{TaskRepository, Task};
use crate::use_cases::process::ProcessUseCase;
use tracing::{info, warn, error};
use std::time::Duration;
use sysinfo::System;
use tokio::time::sleep;
use anyhow::{Result, Context};

pub struct MonitorUseCase {
    settings: Settings,
}

impl MonitorUseCase {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub async fn execute(&self) -> Result<()> {
        let mut monitor = ProcessMonitor::new(self.settings.process_names.clone());
        let recorder = AudioRecorder::new();
        let mut is_recording = false;

        let settings_clone = self.settings.clone();
        
        let watcher = crate::infrastructure::watcher::FileWatcher::new("data/cloud_sync");
        watcher.start()?;

        tokio::spawn(async move {
            let repo = TaskRepository::new("data/tasks.json");
            let prompts = match crate::infrastructure::prompts::Prompts::load() {
                Ok(p) => p,
                Err(e) => {
                    error!("Failed to load prompts in monitor loop: {}", e);
                    return;
                }
            };
            let gemini = GeminiClient::new(
                settings_clone.google_api_key.clone(),
                settings_clone.gemini_model.clone(),
                prompts,
            );
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
                        "health-check high usage cpu={:.1}% memory={:.1}% - Triggering self-restart for OOM protection",
                        cpu, mem_pct
                    );
                    std::process::exit(1);
                } else {
                    info!("health-check cpu={:.1}% memory={:.1}%", cpu, mem_pct);
                }

                sleep(Duration::from_secs(30)).await;
            }
        });

        loop {
            let running = monitor.is_running();
            if running && !is_recording {
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                let path = format!("data/recordings/{}.wav", timestamp);
                if let Err(e) = recorder.start(
                    path,
                    16000,
                    1,
                    self.settings.audio_device.clone(),
                    self.settings.silence_threshold,
                ) {
                    error!("Failed to start recording: {}", e);
                } else {
                    is_recording = true;
                }
            } else if !running && is_recording {
                if let Some(path) = recorder.stop().ok().flatten() {
                    info!("Session recording saved to: {:?}", path);
                    let tasks = TaskRepository::new("data/tasks.json");
                    if let Err(e) = tasks.add("process_session", vec![path.to_string_lossy().to_string()]) {
                        error!("Failed to add task: {}", e);
                    }
                }
                is_recording = false;
            }

            sleep(Duration::from_secs(self.settings.check_interval)).await;
        }
    }
}
