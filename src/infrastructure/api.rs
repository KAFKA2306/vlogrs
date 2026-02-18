use reqwest::Client;
use serde_json::Value;

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

    pub async fn upsert(&self, table: &str, data: &Value) {
        let url = format!("{}/rest/v1/{}", self.url, table);

        let response = self.client
            .post(&url)
            .header("apikey", &self.key)
            .header("Authorization", format!("Bearer {}", self.key))
            .header("Content-Type", "application/json")
            .header("Prefer", "resolution=merge-duplicates")
            .json(data)
            .send()
            .await
            .expect("Failed to send Supabase request");

        if !response.status().is_success() {
            let error = response.text().await.expect("Failed to read error text");
            panic!("Supabase upsert failed: {}", error);
        }
    }
}
