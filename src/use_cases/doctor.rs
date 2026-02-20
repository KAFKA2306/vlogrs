use crate::infrastructure::settings::Settings;
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
    pub fn execute(&self) {
        info!("=== VLog Doctor Checkup ===");
        let ffmpeg = Command::new(crate::domain::constants::FFMPEG_CMD)
            .arg("-version")
            .output();
        match ffmpeg {
            Ok(_) => info!("[OK] FFmpeg is installed"),
            Err(_) => error!("[FAIL] FFmpeg not found in PATH"),
        }
        let sqlite3 = Command::new(crate::domain::constants::SQLITE_CMD)
            .arg("--version")
            .output();
        match sqlite3 {
            Ok(_) => info!("[OK] sqlite3 is installed"),
            Err(_) => error!("[FAIL] sqlite3 not found in PATH"),
        }
        let dirs = [
            crate::domain::constants::RECORDINGS_DIR,
            "data/tasks",
            "data/summaries",
            "data/novels",
            "data/photos",
            crate::domain::constants::LOGS_DIR,
            "journals",
        ];
        for dir in dirs {
            if Path::new(dir).exists() {
                info!("[OK] Directory exists: {}", dir);
            } else {
                warn!("[WARN] Directory missing: {}", dir);
            }
        }
        let _: Settings = Settings::new().unwrap();
        info!("[OK] Configuration (Settings) is valid");
        match crate::infrastructure::prompts::Prompts::load() {
            Ok(_) => info!("[OK] Prompts loaded successfully"),
            Err(e) => panic!("[FATAL] Prompts error: {}", e),
        }
        let _: () = crate::infrastructure::audio::list_devices();
        info!("Checkup complete.");
    }
}
