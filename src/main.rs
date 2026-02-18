pub mod domain;
pub mod infrastructure;
pub mod use_cases;

use clap::{Parser, Subcommand};
use infrastructure::settings::Settings;
use log::info;

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
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let cli: Cli = Cli::parse();
    match cli.command {
        Some(Commands::Monitor) | None => {
            let settings: Settings = Settings::new();
            info!("Starting monitor mode...");
            let use_case: use_cases::monitor::MonitorUseCase =
                use_cases::monitor::MonitorUseCase::new(settings);
            use_case.execute().await;
        }
        Some(Commands::Record) => {
            info!("Starting manual record...");
        }
        Some(Commands::Process { file }) => {
            let settings: Settings = Settings::new();
            info!("Processing file: {}", file);
            let gemini: infrastructure::llm::GeminiClient = infrastructure::llm::GeminiClient::new(
                settings.google_api_key,
                settings.gemini_model,
            );
            let use_case: use_cases::process::ProcessUseCase =
                use_cases::process::ProcessUseCase::new(gemini);
            use_case
                .execute_session(infrastructure::tasks::Task {
                    id: "manual".to_string(),
                    created_at: chrono::Utc::now(),
                    status: "processing".to_string(),
                    task_type: "process_session".to_string(),
                    file_paths: vec![file],
                })
                .await;
        }
        Some(Commands::Novel { date }) => {
            let settings: Settings = Settings::new();
            info!("Building novel for: {}", date);
            let gemini: infrastructure::llm::GeminiClient = infrastructure::llm::GeminiClient::new(
                settings.google_api_key.clone(),
                settings.gemini_model.clone(),
            );
            let image_generator: infrastructure::ai::PythonImageGenerator =
                infrastructure::ai::PythonImageGenerator::new();

            let use_case: use_cases::build_novel::BuildNovelUseCase =
                use_cases::build_novel::BuildNovelUseCase::new(
                    Box::new(gemini),
                    Box::new(image_generator),
                );
            use_case.execute(&date).await;
        }
        Some(Commands::Evaluate { date }) => {
            let settings: Settings = Settings::new();
            info!("Evaluating content for: {}", date);
            let gemini: infrastructure::llm::GeminiClient = infrastructure::llm::GeminiClient::new(
                settings.google_api_key.clone(),
                settings.gemini_model.clone(),
            );

            let supabase: Option<infrastructure::api::SupabaseClient> =
                if !settings.supabase_url.is_empty() {
                    Some(infrastructure::api::SupabaseClient::new(
                        settings.supabase_url,
                        settings.supabase_service_role_key,
                    ))
                } else {
                    None
                };

            let use_case: use_cases::evaluate::EvaluateDailyContentUseCase =
                use_cases::evaluate::EvaluateDailyContentUseCase::new(Box::new(gemini), supabase);
            use_case.execute(&date).await;
        }
        Some(Commands::Sync) => {
            let settings: Settings = Settings::new();
            let use_case: use_cases::sync::SyncUseCase =
                use_cases::sync::SyncUseCase::new(settings);
            use_case.execute().await;
        }
        Some(Commands::Pending) => {
            let use_case: use_cases::pending::PendingUseCase =
                use_cases::pending::PendingUseCase::new();
            use_case.execute().await;
        }
        Some(Commands::Status) => {
            let use_case: use_cases::status::StatusUseCase =
                use_cases::status::StatusUseCase::new();
            use_case.execute().await;
        }
        Some(Commands::Setup) => {
            let use_case: use_cases::setup::SetupUseCase = use_cases::setup::SetupUseCase::new();
            use_case.execute();
        }
    }
}
