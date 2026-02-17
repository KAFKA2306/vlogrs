use crate::infrastructure::api::GeminiClient;
use std::fs;
use std::path::Path;

pub struct Transcriber {
    gemini: GeminiClient,
}

impl Transcriber {
    pub fn new(gemini: GeminiClient) -> Self {
        Self { gemini }
    }

    pub async fn transcribe(&self, file_path: &str) -> anyhow::Result<String> {
        let path = Path::new(file_path);
        let audio_data = fs::read(path)?;

        let mime_type = match path.extension().and_then(|s| s.to_str()) {
            Some("wav") => "audio/wav",
            Some("mp3") => "audio/mpeg",
            Some("m4a") => "audio/mp4",
            Some("flac") => "audio/flac",
            _ => "audio/wav",
        };

        let transcript = self.gemini.transcribe_audio(&audio_data, mime_type).await?;

        let stem = path.file_stem().unwrap().to_str().unwrap();
        let out_path = format!("data/transcripts/{}.txt", stem);
        fs::create_dir_all("data/transcripts")?;
        fs::write(&out_path, &transcript)?;

        Ok(transcript)
    }
}
