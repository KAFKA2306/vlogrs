use crate::infrastructure::llm::GeminiClient;
use std::fs;
use std::path::Path;

pub struct Transcriber {
    gemini: GeminiClient,
}

impl Transcriber {
    pub fn new(gemini: GeminiClient) -> Self {
        Self { gemini }
    }

    pub async fn transcribe(&self, file_path: &str) -> String {
        let path: &Path = Path::new(file_path);
        let audio_data: Vec<u8> = fs::read(path).unwrap();

        let mime_type: &str = match path.extension().and_then(|s| s.to_str()) {
            Some("wav") => "audio/wav",
            Some("mp3") => "audio/mpeg",
            Some("m4a") => "audio/mp4",
            Some("flac") => "audio/flac",
            _ => panic!("Unsupported file type"),
        };

        let transcript: String = self.gemini.transcribe_audio(&audio_data, mime_type).await;

        let stem: &str = path.file_stem().unwrap().to_str().unwrap();
        let out_path: String = format!("data/transcripts/{}.txt", stem);
        fs::create_dir_all("data/transcripts").unwrap();
        fs::write(&out_path, &transcript).unwrap();

        transcript
    }
}
