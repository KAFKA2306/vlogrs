pub mod domain;
pub mod infrastructure;
pub mod use_cases;

use clap::{Parser, Subcommand};
use infrastructure::settings::Settings;
use std::sync::Arc;
use tracing::info;

#[derive(Parser)]
#[command(name = "vlog-rs")]
#[command(about = "Autonomous Life Logger", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Monitor,
    Record,
    Process {
        #[arg(short, long)]
        file: String,
    },
    Novel {
        #[arg(short, long)]
        date: String,
    },
    Evaluate {
        #[arg(short, long)]
        date: String,
    },
    Sync,
    Pending,
    Status,
    Setup,
    Doctor,
    Devices,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    let file_appender = tracing_appender::rolling::daily(
        crate::domain::constants::LOGS_DIR,
        crate::domain::constants::LOG_FILE_NAME,
    );
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(non_blocking)
        .with_ansi(false)
        .json()
        .init();

    std::panic::set_hook(Box::new(|info| {
        let backtrace = std::backtrace::Backtrace::capture();
        tracing::error!("FATAL PANIC: {}\nBacktrace:\n{}", info, backtrace);
    }));

    info!("VLog initialized. System starting...");

    let cli: Cli = Cli::parse();

    let env = Arc::new(infrastructure::fs_utils::LocalEnvironment);

    match cli.command {
        Some(Commands::Monitor) | None => {
            let settings = match Settings::new() {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Initialization failed (Settings): {:?}", e);
                    std::process::exit(1);
                }
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
            let prompts = infrastructure::prompts::Prompts::load().unwrap_or_else(|e| {
                tracing::error!("Failed to load prompts: {:?}", e);
                std::process::exit(1);
            });
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
                env,
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
        Some(Commands::Record) => {
            info!("Starting manual record...");
        }
        Some(Commands::Process { file }) => {
            let settings = match Settings::new() {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Initialization failed (Settings): {:?}", e);
                    std::process::exit(1);
                }
            };
            info!("Processing file: {}", file);
            let prompts = infrastructure::prompts::Prompts::load().unwrap_or_else(|e| {
                tracing::error!("Failed to load prompts: {:?}", e);
                std::process::exit(1);
            });
            let gemini = Arc::new(infrastructure::llm::GeminiClient::new(
                settings.google_api_key,
                settings.gemini_model,
                prompts,
            ));
            let repo = Arc::new(infrastructure::tasks::TaskRepository::new(
                Settings::default_tasks_path(),
            ));
            let event_repo = Arc::new(
                infrastructure::db::EventRepository::new(&settings.db_path.to_string_lossy()).await,
            );
            let use_case =
                use_cases::process::ProcessUseCase::new(gemini.clone(), repo, event_repo, gemini);
            use_case
                .execute_session(&domain::Task {
                    id: "manual".to_string(),
                    created_at: chrono::Utc::now(),
                    status: crate::domain::constants::STATUS_PROCESSING.to_string(),
                    task_type: crate::domain::constants::TASK_TYPE_PROCESS_SESSION.to_string(),
                    file_paths: vec![file],
                })
                .await;
        }
        Some(Commands::Novel { date }) => {
            let settings = match Settings::new() {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Initialization failed (Settings): {:?}", e);
                    std::process::exit(1);
                }
            };
            info!("Building novel for: {}", date);
            let prompts = infrastructure::prompts::Prompts::load().unwrap_or_else(|e| {
                tracing::error!("Failed to load prompts: {:?}", e);
                std::process::exit(1);
            });
            let gemini = infrastructure::llm::GeminiClient::new(
                settings.google_api_key.clone(),
                settings.gemini_model.clone(),
                prompts,
            );
            let image_generator = infrastructure::PythonImageGenerator::new();

            let use_case = use_cases::build_novel::BuildNovelUseCase::new(
                Box::new(gemini.clone()),
                Box::new(gemini),
                Box::new(image_generator),
            );
            use_case.execute(&date).await;
        }
        Some(Commands::Evaluate { date }) => {
            let settings = match Settings::new() {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Initialization failed (Settings): {:?}", e);
                    std::process::exit(1);
                }
            };
            info!("Evaluating content for: {}", date);
            let prompts = infrastructure::prompts::Prompts::load().unwrap_or_else(|e| {
                tracing::error!("Failed to load prompts: {:?}", e);
                std::process::exit(1);
            });
            let gemini = infrastructure::llm::GeminiClient::new(
                settings.google_api_key.clone(),
                settings.gemini_model.clone(),
                prompts,
            );

            let supabase = if !settings.supabase_url.is_empty() {
                Some(infrastructure::api::SupabaseClient::new(
                    settings.supabase_url,
                    settings.supabase_service_role_key,
                ))
            } else {
                None
            };

            let use_case =
                use_cases::evaluate::EvaluateDailyContentUseCase::new(Box::new(gemini), supabase);
            use_case.execute(&date).await;
        }
        Some(Commands::Sync) => {
            let settings = match Settings::new() {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Initialization failed (Settings): {:?}", e);
                    std::process::exit(1);
                }
            };
            let use_case = use_cases::sync::SyncUseCase::new(settings);
            use_case.execute().await;
        }
        Some(Commands::Pending) => {
            let use_case = use_cases::pending::PendingUseCase::new();
            use_case.execute().await;
        }
        Some(Commands::Status) => {
            let use_case = use_cases::status::StatusUseCase::new();
            use_case.execute().await;
        }
        Some(Commands::Setup) => {
            let use_case =
                use_cases::SetupUseCase::new(Box::new(infrastructure::fs_utils::LocalEnvironment));
            use_case.execute();
        }
        Some(Commands::Doctor) => {
            let use_case = use_cases::doctor::DoctorUseCase::new();
            use_case.execute();
        }
        Some(Commands::Devices) => {
            infrastructure::audio::AudioRecorder::list_devices();
        }
    }

    drop(_guard);
}
