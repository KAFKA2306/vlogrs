use crate::domain::{Curator, Evaluation, Novelizer};
use crate::infrastructure::prompts::Prompts;
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use serde_json::{json, Value};
use tokio::time::{sleep, Duration};
use tracing::warn;

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
        let max_retries = 5;
        let base_delay_ms = 500;
        let cap_delay_ms = 8000;

        for attempt in 0..=max_retries {
            let resp = match self.client.post(url).json(&body).send().await {
                Ok(r) => r,
                Err(e) => {
                    if attempt == max_retries {
                        anyhow::bail!("LLM request failed after {} retries: {}", max_retries, e);
                    }
                    self.backoff(attempt, base_delay_ms, cap_delay_ms).await;
                    continue;
                }
            };

            let text = resp.text().await.context("Failed to read response text")?;

            let parsed: Value = match serde_json::from_str(&text) {
                Ok(p) => p,
                Err(e) => {
                    if attempt == max_retries {
                        anyhow::bail!(
                            "Failed to parse LLM response after {} retries: {}",
                            max_retries,
                            e
                        );
                    }
                    self.backoff(attempt, base_delay_ms, cap_delay_ms).await;
                    continue;
                }
            };

            if let Some(content) = parsed["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                return Ok(content.to_string());
            }

            if attempt == max_retries {
                anyhow::bail!("LLM response bad format: {:?}", parsed);
            }
            self.backoff(attempt, base_delay_ms, cap_delay_ms).await;
        }

        anyhow::bail!(
            "LLM request failed to return content after {} retries",
            max_retries
        )
    }

    async fn backoff(&self, attempt: u32, base: u64, cap: u64) {
        let exp = base.saturating_mul(2u64.saturating_pow(attempt));
        let backoff = exp.min(cap);
        let jitter = (chrono::Utc::now().timestamp_subsec_millis() % 250) as u64;
        let wait = backoff + jitter;
        warn!("LLM request retry={} wait={}ms", attempt + 1, wait);
        sleep(Duration::from_millis(wait)).await;
    }
}

#[async_trait::async_trait]
impl Novelizer for GeminiClient {
    async fn generate_chapter(&self, summary: &str, context: &str) -> String {
        let template = &self.prompts.novelizer.template;
        let prompt = template
            .replace("{novel_so_far}", context)
            .replace("{today_summary}", summary);
        self.generate_content(&prompt).await.unwrap_or_else(|e| {
            warn!("Novel generation failed: {}", e);
            "Generation failed.".to_string()
        })
    }
}

#[async_trait::async_trait]
impl Curator for GeminiClient {
    async fn evaluate(&self, summary: &str, novel: &str) -> Evaluation {
        let template = &self.prompts.curator.evaluate;
        let prompt = template
            .replace("{summary}", summary)
            .replace("{novel}", novel);

        let content = self.generate_content(&prompt).await.unwrap_or_default();

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

