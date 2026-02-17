use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub status: String,
    pub task_type: String,
    pub file_paths: Vec<String>,
}

pub struct TaskRepository {
    path: String,
}

impl TaskRepository {
    pub fn new(path: &str) -> Self {
        let repo = Self {
            path: path.to_string(),
        };
        repo.ensure_file();
        repo
    }

    fn ensure_file(&self) {
        if !Path::new(&self.path).exists() {
            fs::write(&self.path, "[]").ok();
        }
    }

    pub fn add(&self, task_type: &str, file_paths: Vec<String>) -> anyhow::Result<Task> {
        let mut tasks = self.load()?;
        let task = Task {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            status: "pending".to_string(),
            task_type: task_type.to_string(),
            file_paths,
        };
        tasks.push(task.clone());
        self.save(&tasks)?;
        Ok(task)
    }

    pub fn load(&self) -> anyhow::Result<Vec<Task>> {
        let content = fs::read_to_string(&self.path)?;
        let tasks: Vec<Task> = serde_json::from_str(&content)?;
        Ok(tasks)
    }

    fn save(&self, tasks: &Vec<Task>) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(tasks)?;
        fs::write(&self.path, content)?;
        Ok(())
    }
}
