use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub status: String,
    pub task_type: String,
    pub file_paths: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SourceType {
    WindowsAudio,
    WindowsActivity,
    UbuntuMonitor,
    System,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LifeEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub source: SourceType,
    pub payload: serde_json::Value,
}
impl LifeEvent {
    pub fn new(source: SourceType, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::now_v7(),
            timestamp: Utc::now(),
            source,
            payload,
        }
    }
}
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
    async fn summarize_session(&self, transcript: &str, activities: &str) -> String;
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
pub trait AudioRecorder: Send + Sync {
    fn start(
        &self,
        output_path: std::path::PathBuf,
        sample_rate: u32,
        channels: u16,
        device_name: Option<String>,
        silence_threshold: f32,
    );
    fn stop(&self) -> Option<std::path::PathBuf>;
}
pub trait ProcessMonitor: Send + Sync {
    fn is_running(&mut self) -> bool;
}
pub trait TaskRepository: Send + Sync {
    fn add(&self, task_type: &str, file_paths: Vec<String>) -> Task;
    fn load(&self) -> Vec<Task>;
    fn update_status(&self, id: &str, status: &str);
}
#[async_trait::async_trait]
pub trait ContentGenerator: Send + Sync {
    async fn generate_content(&self, prompt: &str) -> String;
    async fn transcribe(&self, file_path: &str) -> String;
}
pub trait FileWatcher: Send + Sync {
    fn start(&self);
}
#[async_trait::async_trait]
pub trait EventRepository: Send + Sync {
    async fn save(&self, event: &LifeEvent);
    async fn find_by_timerange(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Vec<LifeEvent>;
}
