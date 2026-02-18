use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub status: String,
    pub task_type: String,
    pub file_paths: Vec<String>,
}
