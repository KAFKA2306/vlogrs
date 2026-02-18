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
}

#[derive(Debug, Deserialize, Clone)]
pub struct AudioSettings {
    pub device_name: Option<String>,
    pub silence_threshold: f32,
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
    pub silence_threshold: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Settings {
    pub fn new() -> Self {
        let s = Config::builder()
            .set_default("process.check_interval", 5)
            .expect("Failed to set default check_interval")
            .set_default("process.names", "VRChat")
            .expect("Failed to set default process.names")
            .set_default("paths.recording_dir", "data/recordings")
            .expect("Failed to set default recording_dir")
            .set_default("audio.silence_threshold", 0.02)
            .expect("Failed to set default silence_threshold")
            .add_source(File::with_name("data/config").required(false))
            .add_source(Environment::default().separator("__"))
            .build()
            .expect("Failed to build configuration");

        let raw: RawSettings = s.try_deserialize().expect("Failed to deserialize configuration");

        Self {
            google_api_key: env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY must be set"),
            gemini_model: env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3-flash".to_string()),
            supabase_url: env::var("SUPABASE_URL").expect("SUPABASE_URL must be set"),
            supabase_service_role_key: env::var("SUPABASE_SERVICE_ROLE_KEY").expect("SUPABASE_SERVICE_ROLE_KEY must be set"),
            check_interval: raw.process.check_interval,
            process_names: raw.process.names.split(',').map(|s| s.trim().to_string()).collect(),
            recording_dir: PathBuf::from(raw.paths.recording_dir),
            audio_device: raw.audio.device_name,
            silence_threshold: raw.audio.silence_threshold,
        }
    }
}
