use anyhow::{Result, Context};
use std::process::Command;
use std::path::Path;
use tracing::{info, error, warn};
use crate::infrastructure::settings::Settings;

pub struct DoctorUseCase;

impl DoctorUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self) -> Result<()> {
        info!("=== VLog Doctor Checkup ===");

        // 1. Check FFmpeg
        let ffmpeg = Command::new("ffmpeg").arg("-version").output();
        match ffmpeg {
            Ok(_) => info!("[OK] FFmpeg is installed"),
            Err(_) => error!("[FAIL] FFmpeg not found in PATH"),
        }

        // 2. Check Directories
        let dirs = ["data/recordings", "data/tasks", "data/summaries", "data/novels", "data/photos"];
        for dir in dirs {
            if Path::new(dir).exists() {
                info!("[OK] Directory exists: {}", dir);
            } else {
                warn!("[WARN] Directory missing: {}", dir);
            }
        }

        // 3. Check Settings
        match Settings::new() {
            Ok(_) => info!("[OK] Configuration (Settings) is valid"),
            Err(e) => error!("[FAIL] Configuration error: {}", e),
        }

        // 4. Check Prompts
        match crate::infrastructure::prompts::Prompts::load() {
            Ok(_) => info!("[OK] Prompts loaded successfully"),
            Err(e) => error!("[FAIL] Prompts error: {}", e),
        }

        info!("Checkup complete.");
        Ok(())
    }
}
