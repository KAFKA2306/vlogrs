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
    pub async fn upsert(&self, table: &str, data: &Value) -> anyhow::Result<()> {
        let url = format!("{}/rest/v1/{}", self.url, table);
        let response = self
            .client
            .post(&url)
            .header("apikey", &self.key)
            .header("Authorization", format!("Bearer {}", self.key))
            .header("Content-Type", "application/json")
            .header("Prefer", "resolution=merge-duplicates")
            .json(data)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Supabase request failed: {}", e))?;
        if !response.status().is_success() {
            let status = response.status();
            let error = response.text().await.unwrap();
            anyhow::bail!("Supabase upsert failed with status {}: {}", status, error);
        }
        Ok(())
    }
}
