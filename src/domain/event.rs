use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
