use crate::domain::{Curator, Evaluation, Novelizer};
use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use serde_json::{json, Value};

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
            "https://generativelanguage.googleapis.com/v1beta/api/models/{}:generateContent?key={}",
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
                        "text": "Please transcribe this audio exactly as it is spoken. Output only the transcript text."
                    }
                ]
            }]
        });

        self.post_and_parse(&url, body).await
    }

    async fn post_and_parse(&self, url: &str, body: Value) -> String {
        let response: String = self
            .client
            .post(url)
            .json(&body)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let json: Value = serde_json::from_str(&response).unwrap();

        json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap()
            .to_string()
    }
}

#[async_trait::async_trait]
impl Novelizer for GeminiClient {
    async fn generate_chapter(&self, summary: &str, context: &str) -> String {
        let prompt: String = format!(
            "これまでのあらすじ:\n{}\n\n今回の出来事要約:\n{}\n\nこれらを元に、日常の1ページを綴るライフログとして、情緒豊かな小説の1章を執筆してください。デジタル（VR、SNS等）か物理（散歩、会議、外出等）かを問わず、その時の情景、音、感情、会話のニュアンスが伝わるような洗練された表現を心がけてください。",
            context, summary
        );
        self.generate_content(&prompt).await
    }
}

#[async_trait::async_trait]
impl Curator for GeminiClient {
    async fn evaluate(&self, summary: &str, novel: &str) -> Evaluation {
        let prompt: String = format!(
            "以下の要約と小説の内容を比較し、評価してください。JSON形式で出力してください。\n\n要約:\n{}\n\n小説:\n{}\n\n期待する形式:\n{{ \"faithfulness_score\": 1-5, \"quality_score\": 1-5, \"reasoning\": \"評価理由\" }}",
            summary, novel
        );
        let content: String = self.generate_content(&prompt).await;

        let cleaned: &str = content
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();

        serde_json::from_str(cleaned).unwrap()
    }
}
