use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
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
    path: PathBuf,
}

impl TaskRepository {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let repo = Self {
            path: path.into(),
        };
        repo.ensure_file();
        repo
    }

    fn ensure_file(&self) {
        if !self.path.exists() {
            fs::write(&self.path, "[]").expect("Failed to create initial tasks file");
        }
    }

    pub fn add(&self, task_type: &str, file_paths: Vec<String>) -> Task {
        let mut tasks = self.load();
        let task = Task {
            id: Uuid::now_v7().to_string(),
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
        let content = fs::read_to_string(&self.path).expect("Failed to read tasks file");
        serde_json::from_str(&content).expect("Failed to parse tasks file")
    }

    pub fn update_status(&self, id: &str, status: &str) {
        let mut tasks = self.load();
        if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
            task.status = status.to_string();
            self.save(&tasks);
        } else {
             panic!("Task with id {} not found", id);
        }
    }

    fn save(&self, tasks: &[Task]) {
        let content = serde_json::to_string_pretty(tasks).expect("Failed to serialize tasks");
        let tmp_path = self.path.with_extension("tmp");
        fs::write(&tmp_path, content).expect("Failed to write temporary tasks file");
        fs::rename(&tmp_path, &self.path).expect("Failed to rename tasks file");
    }
}
