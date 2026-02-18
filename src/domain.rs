use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Evaluation {
    pub faithfulness_score: u8,
    pub quality_score: u8,
    pub reasoning: String,
}

#[async_trait::async_trait]
pub trait Novelizer: Send + Sync {
    async fn generate_chapter(&self, summary: &str, context: &str) -> String;
}

#[async_trait::async_trait]
pub trait Curator: Send + Sync {
    async fn evaluate(&self, summary: &str, novel: &str) -> Evaluation;
}

#[async_trait::async_trait]
pub trait ImageGenerator: Send + Sync {
    async fn generate(&self, prompt: &str, output_path: &str);
}

pub mod constants;

pub trait Environment: Send + Sync {
    fn ensure_directories(&self);
    fn ensure_config(&self);
}
