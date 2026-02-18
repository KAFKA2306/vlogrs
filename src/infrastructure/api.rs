use reqwest::Client;

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

    pub async fn upsert(&self, table: &str, data: serde_json::Value) {
        let url: String = format!("{}/rest/v1/{}", self.url, table);

        let resp: reqwest::Response = self
            .client
            .post(&url)
            .header("apikey", &self.key)
            .header("Authorization", format!("Bearer {}", self.key))
            .header("Content-Type", "application/json")
            .header("Prefer", "resolution=merge-duplicates")
            .json(&data)
            .send()
            .await
            .unwrap();

        if !resp.status().is_success() {
            let error_text: String = resp.text().await.unwrap();
            panic!("Supabase upsert failed: {}", error_text);
        }
    }
}
