use crate::domain::{Curator, Evaluation, Novelizer};
use crate::infrastructure::prompts::Prompts;
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use serde_json::{json, Value};

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

    pub async fn generate_content(&self, prompt: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let body = json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }]
        });

        self.post_and_parse(&url, body).await
    }

    pub async fn transcribe_audio(&self, audio_data: &[u8], mime_type: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let base64_audio = general_purpose::STANDARD.encode(audio_data);

        let body = json!({
            "contents": [{
                "parts": [
                    {
                        "inline_data": {
                            "mime_type": mime_type,
                            "data": base64_audio
                        }
                    },
                    {
                        "text": "Using the audio, write strict dictation. Output only the transcript text."
                    }
                ]
            }]
        });

        self.post_and_parse(&url, body).await
    }

    async fn post_and_parse(&self, url: &str, body: Value) -> Result<String> {
        let resp = self.client.post(url).json(&body).send().await?;
        let text = resp.text().await.context("Failed to read response text")?;
        let parsed: Value = serde_json::from_str(&text)?;

        if let Some(content) = parsed["candidates"][0]["content"]["parts"][0]["text"].as_str() {
            return Ok(content.to_string());
        }

        anyhow::bail!("LLM response bad format: {:?}", parsed)
    }
}

#[async_trait::async_trait]
impl Novelizer for GeminiClient {
    async fn generate_chapter(&self, summary: &str, context: &str) -> String {
        let template = &self.prompts.novelizer.template;
        let prompt = template
            .replace("{novel_so_far}", context)
            .replace("{today_summary}", summary);
        self.generate_content(&prompt)
            .await
            .expect("Failed to generate chapter content")
    }
}

#[async_trait::async_trait]
impl Curator for GeminiClient {
    async fn evaluate(&self, summary: &str, novel: &str) -> Evaluation {
        let template = &self.prompts.curator.evaluate;
        let prompt = template
            .replace("{summary}", summary)
            .replace("{novel}", novel);

        let content = self
            .generate_content(&prompt)
            .await
            .expect("Failed to generate evaluation content");

        let cleaned = content
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();

        serde_json::from_str(cleaned).unwrap_or_else(|_| Evaluation {
            faithfulness_score: 0,
            quality_score: 0,
            reasoning: "Evaluation failed".to_string(),
        })
    }
}
