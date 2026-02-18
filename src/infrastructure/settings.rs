use config::{Config, Environment, File};
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
pub struct AudioSettings {
    pub device_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RawSettings {
    pub process: ProcessSettings,
    pub paths: PathSettings,
    pub audio: AudioSettings,
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
    pub audio_device: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Settings {
    pub fn new() -> Self {
        let s: Config = Config::builder()
            .set_default("process.check_interval", 5)
            .unwrap()
            .set_default("process.names", "VRChat")
            .unwrap()
            .set_default("paths.recording_dir", "data/recordings")
            .unwrap()
            .add_source(File::with_name("data/config").required(false))
            .add_source(Environment::default().separator("__"))
            .build()
            .unwrap();

        let raw: RawSettings = s.try_deserialize().unwrap();

        let google_api_key: String = std::env::var("GOOGLE_API_KEY").unwrap();
        let gemini_model: String = std::env::var("GEMINI_MODEL").unwrap();
        let supabase_url: String = std::env::var("SUPABASE_URL").unwrap();
        let supabase_service_role_key: String = std::env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap();

        let process_names: Vec<String> = raw
            .process
            .names
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Self {
            google_api_key,
            gemini_model,
            supabase_url,
            supabase_service_role_key,
            check_interval: raw.process.check_interval,
            process_names,
            recording_dir: PathBuf::from(raw.paths.recording_dir),
            audio_device: raw.audio.device_name,
        }
    }
}
