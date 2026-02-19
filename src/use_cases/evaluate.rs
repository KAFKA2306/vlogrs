use crate::domain::Curator;
use crate::infrastructure::api::SupabaseClient;
use std::fs;
use std::path::Path;
use tracing::info;

pub struct EvaluateDailyContentUseCase {
    curator: Box<dyn Curator>,
    supabase: Option<SupabaseClient>,
}

impl EvaluateDailyContentUseCase {
    pub fn new(curator: Box<dyn Curator>, supabase: Option<SupabaseClient>) -> Self {
        Self { curator, supabase }
    }

    pub async fn execute(&self, date: &str) {
        let summary_path = crate::domain::constants::SUMMARY_FILE_TEMPLATE.replace("{}", date);
        let novel_path = crate::domain::constants::NOVEL_FILE_TEMPLATE.replace("{}", date);

        if !Path::new(&summary_path).exists() || !Path::new(&novel_path).exists() {
            panic!("Summary or Novel not found for {}", date);
        }

        let summary_text = fs::read_to_string(summary_path).unwrap();
        let novel_text = fs::read_to_string(novel_path).unwrap();

        info!("Evaluating content for {}...", date);
        let result = self.curator.evaluate(&summary_text, &novel_text).await;

        let eval_path = crate::domain::constants::EVALUATION_FILE_TEMPLATE.replace("{}", date);
        if let Some(parent) = Path::new(&eval_path).parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(
            &eval_path,
            serde_json::to_string_pretty(&result).unwrap(),
        )
        .unwrap();
        info!("Evaluation saved to {}", eval_path);

        if let Some(ref supabase) = self.supabase {
            info!("Syncing evaluation to Supabase...");
            let data = serde_json::json!({
                "date": date,
                "target_type": "novel",
                "score": result.quality_score,
                "reasoning": result.reasoning
            });
            supabase.upsert("evaluations", &data).await.unwrap();
        }

        }
}
