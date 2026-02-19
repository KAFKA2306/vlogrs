pub mod event;
pub mod task;

pub use event::{LifeEvent, SourceType};
pub use task::Task;

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
    async fn verify_summary(&self, summary: &str, transcript: &str, activities: &str)
        -> Evaluation;
}

#[async_trait::async_trait]
pub trait ImageGenerator: Send + Sync {
    async fn generate(&self, prompt: &str, output_path: &str) -> anyhow::Result<()>;
}

pub mod constants;

pub trait Environment: Send + Sync {
    fn ensure_directories(&self) -> anyhow::Result<()>;
    fn ensure_config(&self) -> anyhow::Result<()>;
}

pub trait AudioRecorder: Send + Sync {
    fn start(
        &self,
        output_path: std::path::PathBuf,
        sample_rate: u32,
        channels: u16,
        device_name: Option<String>,
        silence_threshold: f32,
    ) -> anyhow::Result<()>;
    fn stop(&self) -> anyhow::Result<Option<std::path::PathBuf>>;
}

pub trait ProcessMonitor: Send + Sync {
    fn is_running(&mut self) -> bool;
}

pub trait TaskRepository: Send + Sync {
    fn add(&self, task_type: &str, file_paths: Vec<String>) -> anyhow::Result<Task>;
    fn load(&self) -> anyhow::Result<Vec<Task>>;
    fn update_status(&self, id: &str, status: &str) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait ContentGenerator: Send + Sync {
    async fn generate_content(&self, prompt: &str) -> anyhow::Result<String>;
    async fn transcribe(&self, file_path: &str) -> anyhow::Result<String>;
}

pub trait FileWatcher: Send + Sync {
    fn start(&self) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait EventRepository: Send + Sync {
    async fn save(&self, event: &LifeEvent) -> anyhow::Result<()>;
    async fn find_by_timerange(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<Vec<LifeEvent>>;
}
