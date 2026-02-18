use crate::domain::{self, Environment};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct LocalEnvironment;

impl Environment for LocalEnvironment {
    fn ensure_directories(&self) {
        for dir in domain::constants::APP_DIRS {
            fs::create_dir_all(dir).expect("Failed to create directory");
        }
    }

    fn ensure_config(&self) {
        let config_path = Path::new(domain::constants::CONFIG_PATH);
        if config_path.exists() {
            println!("Config already exists: {}", config_path.display());
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

        fs::write(config_path, config).expect("Failed to write config");
        println!("Created: {}", config_path.display());
    }
}

impl LocalEnvironment {
    fn prompt_with_default(&self, label: &str, default: &str) -> String {
        print!("{} [{}]: ", label, default);
        io::stdout().flush().expect("Failed to flush stdout");

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

pub fn atomic_write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, content: C) -> io::Result<()> {
    let path = path.as_ref();
    let dir = path.parent().unwrap_or(Path::new("."));

    let mut temp_file = tempfile::Builder::new()
        .prefix("vlog_tmp_")
        .tempfile_in(dir)?;

    temp_file.write_all(content.as_ref())?;
    temp_file.as_file().sync_all()?;
    temp_file.persist(path).map_err(|e| e.error)?;

    let dir_file = std::fs::File::open(dir)?;
    dir_file.sync_all()?;

    Ok(())
}
