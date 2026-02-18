use crate::infrastructure::audio::AudioRecorder;
use crate::infrastructure::llm::GeminiClient;
use crate::infrastructure::process::ProcessMonitor;
use crate::infrastructure::settings::Settings;
use crate::infrastructure::tasks::TaskRepository;
use crate::use_cases::process::ProcessUseCase;
use log::info;
use std::time::Duration;
use tokio::time::sleep;

pub struct MonitorUseCase {
    settings: Settings,
}

impl MonitorUseCase {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub async fn execute(&self) {
        let mut monitor: ProcessMonitor = ProcessMonitor::new(self.settings.process_names.clone());
        let recorder: AudioRecorder = AudioRecorder::new();
        let mut is_recording: bool = false;

        let settings_clone: Settings = self.settings.clone();
        tokio::spawn(async move {
            let repo: TaskRepository = TaskRepository::new("data/tasks.json");
            let gemini: GeminiClient = GeminiClient::new(
                settings_clone.google_api_key.clone(),
                settings_clone.gemini_model.clone(),
            );
            let use_case: ProcessUseCase = ProcessUseCase::new(gemini);

            loop {
                let tasks: Vec<crate::infrastructure::tasks::Task> = repo.load();
                for task in tasks {
                    if task.status == "pending" {
                        repo.update_status(&task.id, "processing");
                        use_case.execute_session(task).await;
                        info!("Task completed");
                    }
                }
                sleep(Duration::from_secs(30)).await;
            }
        });

        loop {
            let running: bool = monitor.is_running();
            if running && !is_recording {
                let timestamp: String = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                let path: String = format!("data/recordings/{}.wav", timestamp);
                std::fs::create_dir_all("data/recordings").unwrap();
                recorder.start(path, 16000, 1, self.settings.audio_device.clone());
                is_recording = true;
            } else if !running && is_recording {
                if let Some(path) = recorder.stop() {
                    info!("Session recording saved to: {}", path);
                    let tasks: TaskRepository = TaskRepository::new("data/tasks.json");
                    tasks.add("process_session", vec![path]);
                }
                is_recording = false;
            }

            sleep(Duration::from_secs(self.settings.check_interval)).await;
        }
    }
}
