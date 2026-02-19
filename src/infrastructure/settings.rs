use config::{Config, Environment, File};
use serde::Deserialize;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct ProcessSettings {
    pub names: String,
    pub check_interval: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PathSettings {
    pub recording_dir: String,
    pub db_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AudioSettings {
    pub device_name: Option<String>,
    pub silence_threshold: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TriggerSettings {
    pub start_debounce_secs: u64,
    pub stop_grace_secs: u64,
    pub min_recording_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RawSettings {
    pub process: ProcessSettings,
    pub paths: PathSettings,
    pub audio: AudioSettings,
    pub trigger: TriggerSettings,
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub google_api_key: String,
    pub gemini_model: String,
    pub supabase_url: String,
    pub supabase_service_role_key: String,
    pub check_interval: u64,
    pub process_names: Vec<String>,
    pub recording_dir: PathBuf,
    pub db_path: PathBuf,
    pub audio_device: Option<String>,
    pub silence_threshold: f32,
    pub start_debounce_secs: u64,
    pub stop_grace_secs: u64,
    pub min_recording_secs: u64,
}

impl Settings {
    pub fn new() -> Result<Self, anyhow::Error> {
        let s = Config::builder()
            .set_default(
                "process.check_interval",
                crate::domain::constants::MONITOR_CHECK_INTERVAL_DEFAULT,
            )?
            .set_default(
                "process.names",
                crate::domain::constants::DEFAULT_PROCESS_NAMES,
            )?
            .set_default("paths.recording_dir", crate::domain::constants::APP_DIRS[0])?
            .set_default("paths.db_path", crate::domain::constants::DEFAULT_DB_PATH)?
            .set_default(
                "audio.silence_threshold",
                crate::domain::constants::DEFAULT_SILENCE_THRESHOLD,
            )?
            .set_default(
                "trigger.start_debounce_secs",
                crate::domain::constants::START_DEBOUNCE_SECS_DEFAULT,
            )?
            .set_default(
                "trigger.stop_grace_secs",
                crate::domain::constants::STOP_GRACE_SECS_DEFAULT,
            )?
            .set_default(
                "trigger.min_recording_secs",
                crate::domain::constants::MIN_RECORDING_SECS_DEFAULT,
            )?
            .add_source(File::with_name(crate::domain::constants::CONFIG_PATH).required(false))
            .add_source(Environment::default().separator("__"))
            .build()?;

        let raw: RawSettings = s.try_deserialize()?;

        let google_api_key = env::var("GOOGLE_API_KEY")
            .map_err(|_| anyhow::anyhow!("GOOGLE_API_KEY must be set"))?;

        let gemini_model =
            env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3-flash".to_string());

        let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
        let supabase_service_role_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();

        Ok(Self {
            google_api_key,
            gemini_model,
            supabase_url,
            supabase_service_role_key,
            check_interval: raw.process.check_interval,
            process_names: raw
                .process
                .names
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            recording_dir: Self::translate_path(raw.paths.recording_dir),
            db_path: Self::translate_path(raw.paths.db_path),
            audio_device: raw.audio.device_name,
            silence_threshold: raw.audio.silence_threshold,
            start_debounce_secs: raw.trigger.start_debounce_secs,
            stop_grace_secs: raw.trigger.stop_grace_secs,
            min_recording_secs: raw.trigger.min_recording_secs,
        })
    }

    pub fn default_tasks_path() -> PathBuf {
        PathBuf::from(crate::domain::constants::TASKS_PATH)
    }

    fn translate_path(path: String) -> PathBuf {
        if cfg!(windows) && path.starts_with("/mnt/") {
            let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            // parts will be ["mnt", "m", "Data", "vlog", ...]
            if parts.len() >= 2 && parts[0] == "mnt" {
                let drive = parts[1].to_uppercase();
                let rest = parts[2..].join("\\");
                return PathBuf::from(format!("{}:\\{}", drive, rest));
            }
        }
        PathBuf::from(path)
    }
}
