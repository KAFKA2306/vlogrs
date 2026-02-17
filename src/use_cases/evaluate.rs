use crate::infrastructure::ai::Curator;
use crate::infrastructure::api::SupabaseClient;
use log::info;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub struct EvaluateDailyContentUseCase {
    curator: Curator,
    supabase: Option<SupabaseClient>,
}

impl EvaluateDailyContentUseCase {
    pub fn new(curator: Curator, supabase: Option<SupabaseClient>) -> Self {
        Self { curator, supabase }
    }

    pub async fn execute(&self, date: &str) -> anyhow::Result<Value> {
        let summary_path = format!("data/summaries/{}_summary.txt", date);
        let novel_path = format!("data/novels/{}.md", date);

        if !Path::new(&summary_path).exists() || !Path::new(&novel_path).exists() {
            anyhow::bail!("Summary or Novel not found for {}", date);
        }

        let summary_text = fs::read_to_string(summary_path)?;
        let novel_text = fs::read_to_string(novel_path)?;

        info!("Evaluating content for {}...", date);
        let result = self.curator.evaluate(&summary_text, &novel_text).await?;

        let eval_path = format!("data/evaluations/{}.json", date);
        fs::create_dir_all("data/evaluations")?;
        fs::write(&eval_path, serde_json::to_string_pretty(&result)?)?;
        info!("Evaluation saved to {}", eval_path);

        if let Some(ref supabase) = self.supabase {
            info!("Syncing evaluation to Supabase...");
            let data = serde_json::json!({
                "date": date,
                "target_type": "novel",
                "score": result["quality_score"],
                "reasoning": result["reasoning"]
            });
            supabase.upsert("evaluations", data).await?;
        }

        Ok(result)
    }
}
