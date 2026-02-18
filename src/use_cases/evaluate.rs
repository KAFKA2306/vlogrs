use crate::domain::{Curator, Evaluation};
use crate::infrastructure::api::SupabaseClient;
use log::info;
use std::fs;
use std::path::Path;

pub struct EvaluateDailyContentUseCase {
    curator: Box<dyn Curator>,
    supabase: Option<SupabaseClient>,
}

impl EvaluateDailyContentUseCase {
    pub fn new(curator: Box<dyn Curator>, supabase: Option<SupabaseClient>) -> Self {
        Self { curator, supabase }
    }

    pub async fn execute(&self, date: &str) -> Evaluation {
        let summary_path: String = format!("data/summaries/{}_summary.txt", date);
        let novel_path: String = format!("data/novels/{}.md", date);

        if !Path::new(&summary_path).exists() || !Path::new(&novel_path).exists() {
            panic!("Summary or Novel not found for {}", date);
        }

        let summary_text: String = fs::read_to_string(summary_path).unwrap();
        let novel_text: String = fs::read_to_string(novel_path).unwrap();

        info!("Evaluating content for {}...", date);
        let result: Evaluation = self.curator.evaluate(&summary_text, &novel_text).await;

        let eval_path: String = format!("data/evaluations/{}.json", date);
        fs::create_dir_all("data/evaluations").unwrap();
        fs::write(&eval_path, serde_json::to_string_pretty(&result).unwrap()).unwrap();
        info!("Evaluation saved to {}", eval_path);

        if let Some(ref supabase) = self.supabase {
            info!("Syncing evaluation to Supabase...");
            let data: serde_json::Value = serde_json::json!({
                "date": date,
                "target_type": "novel",
                "score": result.quality_score,
                "reasoning": result.reasoning
            });
            supabase.upsert("evaluations", data).await;
        }

        result
    }
}
