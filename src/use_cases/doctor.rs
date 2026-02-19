use crate::infrastructure::settings::Settings;
use anyhow::Result;
use std::path::Path;
use std::process::Command;
use tracing::{error, info, warn};

pub struct DoctorUseCase;

impl Default for DoctorUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl DoctorUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self) -> Result<()> {
        info!("=== VLog Doctor Checkup ===");

        let ffmpeg = Command::new("ffmpeg").arg("-version").output();
        match ffmpeg {
            Ok(_) => info!("[OK] FFmpeg is installed"),
            Err(_) => error!("[FAIL] FFmpeg not found in PATH"),
        }

        let sqlite3 = Command::new("sqlite3").arg("--version").output();
        match sqlite3 {
            Ok(_) => info!("[OK] sqlite3 is installed"),
            Err(_) => error!("[FAIL] sqlite3 not found in PATH"),
        }

        let dirs = [
            "data/recordings",
            "data/tasks",
            "data/summaries",
            "data/novels",
            "data/photos",
            "logs",
            "journals",
        ];
        for dir in dirs {
            if Path::new(dir).exists() {
                info!("[OK] Directory exists: {}", dir);
            } else {
                warn!("[WARN] Directory missing: {}", dir);
            }
        }

        match Settings::new() {
            Ok(_) => info!("[OK] Configuration (Settings) is valid"),
            Err(e) => error!("[FAIL] Configuration error: {}", e),
        }

        match crate::infrastructure::prompts::Prompts::load() {
            Ok(_) => info!("[OK] Prompts loaded successfully"),
            Err(e) => error!("[FAIL] Prompts error: {}", e),
        }

        info!("Checkup complete.");
        Ok(())
    }
}
