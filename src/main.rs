pub mod domain;
pub mod infrastructure;
pub mod use_cases;
pub mod models;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vlog-rs")]
#[command(about = "Rust version of VRChat Auto-Diary", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
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
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Record) => {
            println!("Starting record...");
            // TODO: Invoke infrastructure::audio
        }
        Some(Commands::Process { file }) => {
            println!("Processing file: {}", file);
            // TODO: Invoke use_cases::process
        }
        Some(Commands::Sync) => {
            println!("Syncing to Supabase...");
            // TODO: Invoke infrastructure::supabase
        }
        None => {
            println!("Use --help for usage");
        }
    }

    Ok(())
}
