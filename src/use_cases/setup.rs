use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct SetupUseCase;

impl Default for SetupUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl SetupUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self) {
        self.create_dirs();
        self.ensure_config_file();
        println!("Setup complete.");
    }

    fn create_dirs(&self) {
        let dirs = [
            "data/recordings",
            "data/transcripts",
            "data/summaries",
            "data/novels",
            "data/photos",
            "data/archives",
            "data/recordings/partial",
        ];

        for dir in dirs {
            if let Err(err) = fs::create_dir_all(dir) {
                eprintln!("Failed to create {}: {}", dir, err);
            }
        }
    }

    fn ensure_config_file(&self) {
        let config_path = Path::new("data/config.yaml");
        if config_path.exists() {
            println!("Config already exists: data/config.yaml");
            return;
        }

        let process_names = self.prompt_with_default(
            "Process names (comma separated)",
            "VRChat.exe,vrchat,VRChatClient.exe",
        );
        let check_interval = self.prompt_with_default("Check interval seconds", "5");
        let device_name = self.prompt_with_default("Audio device name (blank = default)", "");

        let config = format!(
            "process:\n  names: \"{}\"\n  check_interval: {}\npaths:\n  recording_dir: \"data/recordings\"\naudio:\n  device_name: {}\n",
            process_names,
            check_interval,
            if device_name.is_empty() {
                "null".to_string()
            } else {
                format!("\"{}\"", device_name)
            }
        );

        if let Err(err) = fs::write(config_path, config) {
            eprintln!("Failed to write data/config.yaml: {}", err);
            return;
        }

        println!("Created: data/config.yaml");
    }

    fn prompt_with_default(&self, label: &str, default: &str) -> String {
        print!("{} [{}]: ", label, default);
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return default.to_string();
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            default.to_string()
        } else {
            trimmed.to_string()
        }
    }
}
