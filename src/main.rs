pub mod cli;
pub mod domain;
pub mod infrastructure;
pub mod use_cases;
use clap::{Parser, Subcommand};
#[derive(Parser)]
#[command(name = "vlog-rs")]
#[command(about = "Autonomous Life Logger", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}
#[derive(Subcommand)]
enum Commands {
    Monitor {
        #[arg(long, default_value_t = false)]
        no_worker: bool,
    },
    Worker,
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
        let backtrace: std::backtrace::Backtrace = std::backtrace::Backtrace::capture();
        tracing::error!("FATAL PANIC: {}\nBacktrace:\n{}", info, backtrace);
    }));
    let cli: Cli = Cli::parse();
    match cli.command {
        Some(Commands::Monitor { no_worker }) => {
            cli::monitor::run(no_worker).await;
        }
        Some(Commands::Worker) => {
            cli::worker::run().await;
        }
        None => {
            cli::monitor::run(false).await;
        }
        Some(Commands::Record) => {
            cli::record::run().await.unwrap();
        }
        Some(Commands::Process { file }) => {
            cli::process::run(file).await;
        }
        Some(Commands::Novel { date }) => {
            cli::novel::run(date).await;
        }
        Some(Commands::Evaluate { date }) => {
            cli::evaluate::run(date).await;
        }
        Some(Commands::Sync) => {
            cli::sync::run().await;
        }
        Some(Commands::Pending) => {
            use_cases::pending::PendingUseCase::new().execute().await;
        }
        Some(Commands::Status) => {
            cli::status::run().await;
        }
        Some(Commands::Setup) => {
            cli::setup::run();
        }
        Some(Commands::Doctor) => {
            cli::doctor::run();
        }
        Some(Commands::Devices) => {
            infrastructure::audio::list_devices();
        }
    }
    drop(_guard);
}
