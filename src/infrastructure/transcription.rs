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

    pub async fn transcribe(&self, file_path: impl AsRef<Path>) -> String {
        let path = file_path.as_ref();
        
        // Normalize to temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("vlog_norm_{}.wav", uuid::Uuid::new_v4()));
        
        let path_buf = path.to_path_buf();
        if let Err(e) = crate::infrastructure::audio::AudioRecorder::normalize_audio(&path_buf, &temp_file) {
            log::warn!("Audio normalization failed for {:?}: {}, trying original file", path, e);
             // Verify if original is readable, else panic
        }
        
        let (read_path, mime) = if temp_file.exists() {
            (temp_file.clone(), "audio/wav")
        } else {
             (path.to_path_buf(), match path.extension().and_then(|s| s.to_str()) {
                Some("wav") => "audio/wav",
                Some("mp3") => "audio/mpeg",
                Some("m4a") => "audio/mp4",
                Some("flac") => "audio/flac",
                _ => panic!("Unsupported file type: {:?}", path),
            })
        };

        let audio_data = fs::read(&read_path).expect("Failed to read audio file");
        let mime_type = mime;
        
        let transcript = self.gemini.transcribe_audio(&audio_data, mime_type).await;
        
        if temp_file.exists() {
            let _ = fs::remove_file(temp_file);
        }

        let stem = path.file_stem().expect("Invalid file name").to_str().expect("Invalid unicode in filename");
        let out_path = format!("data/transcripts/{}.txt", stem);
        fs::create_dir_all("data/transcripts").expect("Failed to create transcripts directory");
        fs::write(&out_path, &transcript).expect("Failed to write transcript file");

        transcript
    }
}
