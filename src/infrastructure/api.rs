use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;

#[derive(Clone)]
pub struct GeminiClient {
    api_key: String,
    model: String,
    client: Client,
}

impl GeminiClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: Client::new(),
        }
    }

    pub async fn generate_content(&self, prompt: &str) -> anyhow::Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let body = serde_json::json!({
            "contents": [{
                "parts": [{"text": prompt}]
            }]
        });

        self.post_and_parse(&url, body).await
    }

    pub async fn transcribe_audio(
        &self,
        audio_data: &[u8],
        mime_type: &str,
    ) -> anyhow::Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let base64_audio = general_purpose::STANDARD.encode(audio_data);

        let body = serde_json::json!({
            "contents": [{
                "parts": [
                    {
                        "inline_data": {
                            "mime_type": mime_type,
                            "data": base64_audio
                        }
                    },
                    {
                        "text": "Please transcribe this audio exactly as it is spoken. Output only the transcript text."
                    }
                ]
            }]
        });

        self.post_and_parse(&url, body).await
    }

    async fn post_and_parse(&self, url: &str, body: serde_json::Value) -> anyhow::Result<String> {
        let resp = self.client.post(url).json(&body).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            anyhow::bail!("Gemini API error: {}", error_text);
        }

        let json: serde_json::Value = resp.json().await?;

        let text = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Gemini response: {:?}", json))?
            .to_string();

        Ok(text)
    }
}

pub struct SupabaseClient {
    url: String,
    key: String,
    client: Client,
}

impl SupabaseClient {
    pub fn new(url: String, key: String) -> Self {
        Self {
            url,
            key,
            client: Client::new(),
        }
    }

    pub async fn upsert(&self, table: &str, data: serde_json::Value) -> anyhow::Result<()> {
        let url = format!("{}/rest/v1/{}", self.url, table);

        let resp = self
            .client
            .post(&url)
            .header("apikey", &self.key)
            .header("Authorization", format!("Bearer {}", self.key))
            .header("Content-Type", "application/json")
            .header("Prefer", "resolution=merge-duplicates")
            .json(&data)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            anyhow::bail!("Supabase upsert failed: {}", error_text);
        }

        Ok(())
    }
}
