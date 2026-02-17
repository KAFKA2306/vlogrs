pub mod domain;
pub mod infrastructure;
pub mod models;
pub mod use_cases;

use clap::{Parser, Subcommand};
use infrastructure::settings::Settings;
use log::info;

#[derive(Parser)]
#[command(name = "vlog-rs")]
#[command(about = "Rust version of VRChat Auto-Diary", long_about = None)]
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
            let use_case = use_cases::monitor::MonitorUseCase::new(settings);
            use_case.execute().await?;
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
        Some(Commands::Novel { date }) => {
            info!("Building novel for: {}", date);
            let gemini = infrastructure::api::GeminiClient::new(
                settings.google_api_key.clone(),
                settings.gemini_model.clone(),
            );
            let novelizer = infrastructure::ai::Novelizer::new(gemini);
            let image_generator = infrastructure::ai::ImageGenerator::new();
            let use_case =
                use_cases::build_novel::BuildNovelUseCase::new(novelizer, image_generator);
            use_case.execute(&date).await?;
        }
        Some(Commands::Evaluate { date }) => {
            info!("Evaluating content for: {}", date);
            let gemini = infrastructure::api::GeminiClient::new(
                settings.google_api_key.clone(),
                settings.gemini_model.clone(),
            );
            let curator = infrastructure::ai::Curator::new(gemini);
            let supabase = if !settings.supabase_url.is_empty() {
                Some(infrastructure::api::SupabaseClient::new(
                    settings.supabase_url,
                    settings.supabase_service_role_key,
                ))
            } else {
                None
            };
            let use_case = use_cases::evaluate::EvaluateDailyContentUseCase::new(curator, supabase);
            use_case.execute(&date).await?;
        }
        Some(Commands::Sync) => {
            let use_case = use_cases::sync::SyncUseCase::new(settings);
            use_case.execute().await?;
        }
    }

    Ok(())
}
