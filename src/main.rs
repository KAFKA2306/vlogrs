pub mod domain;
pub mod infrastructure;
pub mod models;
pub mod use_cases;

use clap::{Parser, Subcommand};
use infrastructure::process::ProcessMonitor;
use infrastructure::settings::Settings;
use log::{error, info};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Parser)]
#[command(name = "vlog-rs")]
#[command(about = "Rust version of VRChat Auto-Diary", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 監視モード
    Monitor,
    /// 手動録音
    Record,
    /// 1ファイルの処理
    Process {
        #[arg(short, long)]
        file: String,
    },
    /// Supabase同期
    Sync,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let cli = Cli::parse();
    let settings = Settings::new()?;

    match cli.command {
        Some(Commands::Monitor) | None => {
            info!("Starting monitor mode...");
            let mut monitor = ProcessMonitor::new(settings.process_names.clone());
            let recorder = infrastructure::audio::AudioRecorder::new();
            let mut is_recording = false;

            let settings_clone = settings.clone();
            tokio::spawn(async move {
                let repo = infrastructure::tasks::TaskRepository::new("data/tasks.json");
                let gemini = infrastructure::api::GeminiClient::new(
                    settings_clone.google_api_key.clone(),
                    settings_clone.gemini_model.clone(),
                );
                let use_case = use_cases::process::ProcessUseCase::new(gemini);

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
                        let tasks = infrastructure::tasks::TaskRepository::new("data/tasks.json");
                        if let Err(e) = tasks.add("process_session", vec![path]) {
                            error!("Failed to add task: {}", e);
                        }
                    }
                    is_recording = false;
                }

                sleep(Duration::from_secs(settings.check_interval)).await;
            }
        }
        Some(Commands::Record) => {
            info!("Starting manual record...");
        }
        Some(Commands::Process { file }) => {
            info!("Processing file: {}", file);
            let gemini = infrastructure::api::GeminiClient::new(
                settings.google_api_key,
                settings.gemini_model,
            );
            let use_case = use_cases::process::ProcessUseCase::new(gemini);
            use_case
                .execute_session(infrastructure::tasks::Task {
                    id: "manual".to_string(),
                    created_at: chrono::Utc::now(),
                    status: "processing".to_string(),
                    task_type: "process_session".to_string(),
                    file_paths: vec![file],
                })
                .await?;
        }
        Some(Commands::Sync) => {
            let client = infrastructure::api::SupabaseClient::new(
                settings.supabase_url,
                settings.supabase_service_role_key,
            );
            let summaries = std::fs::read_dir("data/summaries")?;
            for entry in summaries {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                    let content = std::fs::read_to_string(&path)?;
                    let date_str = path
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .split('_')
                        .next()
                        .unwrap();
                    let data = serde_json::json!({
                        "file_path": path.to_string_lossy(),
                        "date": date_str,
                        "content": content,
                        "tags": ["summary"]
                    });
                    client.upsert("daily_entries", data).await?;
                    info!("Synced {}", path.display());
                }
            }
        }
    }

    Ok(())
}
