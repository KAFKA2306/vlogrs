use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct ProcessSettings {
    pub names: String,
    pub check_interval: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PathSettings {
    pub recording_dir: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RawSettings {
    pub process: ProcessSettings,
    pub paths: PathSettings,
}

#[derive(Clone)]
pub struct Settings {
    pub google_api_key: String,
    pub gemini_model: String,
    pub supabase_url: String,
    pub supabase_service_role_key: String,
    pub check_interval: u64,
    pub process_names: Vec<String>,
    pub recording_dir: PathBuf,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .set_default("process.check_interval", 5)?
            .set_default("process.names", "VRChat")?
            .set_default("paths.recording_dir", "data/recordings")?
            .add_source(File::with_name("data/config").required(false))
            .add_source(Environment::default().separator("__"))
            .build()?;

        let raw: RawSettings = s.try_deserialize()?;

        let google_api_key = std::env::var("GOOGLE_API_KEY").unwrap_or_default();
        let gemini_model =
            std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.0-flash".to_string());
        let supabase_url = std::env::var("SUPABASE_URL").unwrap_or_default();
        let supabase_service_role_key =
            std::env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();

        let process_names = raw
            .process
            .names
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Self {
            google_api_key,
            gemini_model,
            supabase_url,
            supabase_service_role_key,
            check_interval: raw.process.check_interval,
            process_names,
            recording_dir: PathBuf::from(raw.paths.recording_dir),
        })
    }
}
