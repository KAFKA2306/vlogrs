use crate::infrastructure::api::GeminiClient;
use crate::infrastructure::audio::AudioRecorder;
use crate::infrastructure::process::ProcessMonitor;
use crate::infrastructure::settings::Settings;
use crate::infrastructure::tasks::TaskRepository;
use crate::use_cases::process::ProcessUseCase;
use log::{error, info};
use std::time::Duration;
use tokio::time::sleep;

pub struct MonitorUseCase {
    settings: Settings,
}

impl MonitorUseCase {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub async fn execute(&self) -> anyhow::Result<()> {
        let mut monitor = ProcessMonitor::new(self.settings.process_names.clone());
        let recorder = AudioRecorder::new();
        let mut is_recording = false;

        let settings_clone = self.settings.clone();
        tokio::spawn(async move {
            let repo = TaskRepository::new("data/tasks.json");
            let gemini = GeminiClient::new(
                settings_clone.google_api_key.clone(),
                settings_clone.gemini_model.clone(),
            );
            let use_case = ProcessUseCase::new(gemini);

            loop {
                if let Ok(tasks) = repo.load() {
                    for task in tasks {
                        if task.status == "pending" {
                            if let Err(e) = use_case.execute_session(task).await {
                                error!("Task processing failed: {}", e);
                            } else {
                                info!("Task completed");
                            }
                        }
                    }
                }
                sleep(Duration::from_secs(30)).await;
            }
        });

        loop {
            let running = monitor.is_running();
            if running && !is_recording {
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                let path = format!("data/recordings/{}.wav", timestamp);
                std::fs::create_dir_all("data/recordings")?;
                if let Err(e) = recorder.start(path, 16000, 1) {
                    error!("Failed to start recording: {}", e);
                } else {
                    is_recording = true;
                }
            } else if !running && is_recording {
                if let Some(path) = recorder.stop() {
                    info!("Session recording saved to: {}", path);
                    let tasks = TaskRepository::new("data/tasks.json");
                    if let Err(e) = tasks.add("process_session", vec![path]) {
                        error!("Failed to add task: {}", e);
                    }
                }
                is_recording = false;
            }

            sleep(Duration::from_secs(self.settings.check_interval)).await;
        }
    }
}
