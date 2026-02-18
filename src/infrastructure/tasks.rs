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
        let repo: Self = Self {
            path: path.to_string(),
        };
        repo.ensure_file();
        repo
    }

    fn ensure_file(&self) {
        if !Path::new(&self.path).exists() {
            fs::write(&self.path, "[]").unwrap();
        }
    }

    pub fn add(&self, task_type: &str, file_paths: Vec<String>) -> Task {
        let mut tasks: Vec<Task> = self.load();
        let task: Task = Task {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            status: "pending".to_string(),
            task_type: task_type.to_string(),
            file_paths,
        };
        tasks.push(task.clone());
        self.save(&tasks);
        task
    }

    pub fn load(&self) -> Vec<Task> {
        let content: String = fs::read_to_string(&self.path).unwrap();
        serde_json::from_str(&content).unwrap()
    }

    pub fn update_status(&self, id: &str, status: &str) {
        let mut tasks: Vec<Task> = self.load();
        if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
            task.status = status.to_string();
            self.save(&tasks);
        }
    }

    fn save(&self, tasks: &Vec<Task>) {
        let content: String = serde_json::to_string_pretty(tasks).unwrap();
        fs::write(&self.path, content).unwrap();
    }
}
