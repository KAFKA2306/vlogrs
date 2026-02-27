use crate::domain::{Curator, Evaluation, Novelizer};
use crate::infrastructure::prompts::Prompts;
use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use serde_json::{json, Value};
#[derive(Clone)]
pub struct NoopGemini;
impl Default for NoopGemini {
    fn default() -> Self {
        Self::new()
    }
}

impl NoopGemini {
    pub fn new() -> Self {
        Self
    }
}
#[async_trait::async_trait]
impl crate::domain::ContentGenerator for NoopGemini {
    async fn generate_content(&self, _prompt: &str) -> String {
        "".to_string()
    }
    async fn transcribe(&self, _file_path: &str) -> String {
        "".to_string()
    }
}
#[async_trait::async_trait]
impl Curator for NoopGemini {
    async fn evaluate(&self, _summary: &str, _novel: &str) -> Evaluation {
        Evaluation {
            faithfulness_score: 0,
            quality_score: 0,
            reasoning: "Gemini disabled".to_string(),
        }
    }
    async fn verify_summary(
        &self,
        _summary: &str,
        _transcript: &str,
        _activities: &str,
    ) -> Evaluation {
        Evaluation {
            faithfulness_score: 0,
            quality_score: 0,
            reasoning: "Gemini disabled".to_string(),
        }
    }
    async fn summarize_session(&self, _transcript: &str, _activities: &str) -> String {
        "".to_string()
    }
}
#[derive(Clone)]
pub struct GeminiClient {
    api_key: String,
    model: String,
    client: Client,
    prompts: Prompts,
}
impl GeminiClient {
    pub fn new(api_key: String, model: String, prompts: Prompts) -> Self {
        Self {
            api_key,
            model,
            client: Client::new(),
            prompts,
        }
    }
    pub async fn generate_content(&self, prompt: &str) -> String {
        let url: String = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );
        let body: Value = json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }]
        });
        self.post_and_parse(&url, body).await
    }
    pub async fn transcribe_audio(&self, audio_data: &[u8], mime_type: &str) -> String {
        let url: String = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );
        let base64_audio: String = general_purpose::STANDARD.encode(audio_data);
        let body: Value = json!({
            "contents": [{
                "parts": [
                    {
                        "inline_data": {
                            "mime_type": mime_type,
                            "data": base64_audio
                        }
                    },
                    {
                        "text": &self.prompts.transcription
                    }
                ]
            }]
        });
        self.post_and_parse(&url, body).await
    }
    async fn post_and_parse(&self, url: &str, body: Value) -> String {
        let resp = self.client.post(url).json(&body).send().await.unwrap();
        let text: String = resp.text().await.unwrap();
        let parsed: Value = serde_json::from_str(&text).unwrap();
        if let Some(content) = parsed["candidates"][0]["content"]["parts"][0]["text"].as_str() {
            return content.to_string();
        }
        panic!("LLM response bad format: {:?}", parsed)
    }
}
#[async_trait::async_trait]
impl Novelizer for GeminiClient {
    async fn generate_chapter(&self, summary: &str, context: &str) -> String {
        let template: &String = &self.prompts.novelizer.template;
        let prompt: String = template
            .replace("{novel_so_far}", context)
            .replace("{today_summary}", summary);
        self.generate_content(&prompt).await
    }
}
#[async_trait::async_trait]
impl crate::domain::ContentGenerator for GeminiClient {
    async fn generate_content(&self, prompt: &str) -> String {
        self.generate_content(prompt).await
    }
    async fn transcribe(&self, file_path: &str) -> String {
        let audio_data: Vec<u8> = std::fs::read(file_path).unwrap();
        let ext: &str = std::path::Path::new(file_path)
            .extension()
            .unwrap()
            .to_str()
            .unwrap();
        let mime_type: &str = match ext {
            "wav" => "audio/wav",
            "flac" => "audio/flac",
            "mp3" => "audio/mp3",
            _ => "audio/wav",
        };
        self.transcribe_audio(&audio_data, mime_type).await
    }
}
#[async_trait::async_trait]
impl Curator for GeminiClient {
    async fn evaluate(&self, summary: &str, novel: &str) -> Evaluation {
        let template: &String = &self.prompts.curator.evaluate;
        let prompt: String = template
            .replace("{summary}", summary)
            .replace("{novel}", novel);
        let content: String = self.generate_content(&prompt).await;
        let cleaned: &str = content
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();
        serde_json::from_str(cleaned).unwrap()
    }
    async fn verify_summary(
        &self,
        summary: &str,
        transcript: &str,
        activities: &str,
    ) -> Evaluation {
        let prompt: String = self
            .prompts
            .summary_verification
            .replace("{summary}", summary)
            .replace("{transcript}", transcript)
            .replace("{activities}", activities);
        let content: String = self.generate_content(&prompt).await;
        let cleaned: &str = content
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();
        serde_json::from_str(cleaned).unwrap()
    }
    async fn summarize_session(&self, transcript: &str, activities: &str) -> String {
        let prompt: String = self
            .prompts
            .curator
            .session_summary
            .replace("{transcript}", transcript)
            .replace("{activity_context}", activities);
        self.generate_content(&prompt).await
    }
}
