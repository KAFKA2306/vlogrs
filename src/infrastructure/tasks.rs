use chrono::{DateTime, Utc};
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
        let content = match fs::read_to_string(&self.path) {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        if let Ok(tasks) = serde_json::from_str::<Vec<Task>>(&content) {
            return tasks;
        }

        let mut migrated = vec![];
        let parsed: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(err) => {
                warn!("Failed to parse tasks file: {}", err);
                return vec![];
            }
        };

        if let Some(items) = parsed.as_array() {
            for item in items {
                let id = item
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| Uuid::now_v7().to_string());
                let status = item
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("pending")
                    .to_string();
                let task_type = item
                    .get("task_type")
                    .and_then(Value::as_str)
                    .unwrap_or("legacy")
                    .to_string();

                let mut file_paths = vec![];
                if let Some(paths) = item.get("file_paths").and_then(Value::as_array) {
                    for path in paths {
                        if let Some(s) = path.as_str() {
                            file_paths.push(s.to_string());
                        }
                    }
                }

                migrated.push(Task {
                    id,
                    created_at: Utc::now(),
                    status,
                    task_type,
                    file_paths,
                });
            }
        }

        migrated
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
        let tmp_path = format!("{}.tmp", &self.path);
        fs::write(&tmp_path, content).unwrap();
        fs::rename(&tmp_path, &self.path).unwrap();
    }
}
