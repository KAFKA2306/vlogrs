use crate::domain::Task;
use crate::domain::TaskRepository as TaskRepositoryTrait;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub struct TaskRepository {
    path: PathBuf,
}

impl TaskRepository {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn ensure_file(&self) {
        if !self.path.exists() {
            fs::write(&self.path, "[]").unwrap();
        }
    }

    fn save(&self, tasks: &[Task]) {
        let content = serde_json::to_string_pretty(tasks).unwrap();
        let tmp_path = self.path.with_extension("tmp");
        fs::write(&tmp_path, content).unwrap();
        fs::rename(&tmp_path, &self.path).unwrap();
    }
}

impl TaskRepositoryTrait for TaskRepository {
    fn add(&self, task_type: &str, file_paths: Vec<String>) -> Task {
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

    fn load(&self) -> Vec<Task> {
        self.ensure_file();
        let content = fs::read_to_string(&self.path).unwrap();
        serde_json::from_str(&content).unwrap()
    }

    fn update_status(&self, id: &str, status: &str) {
        let mut tasks = self.load();
        if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
            task.status = status.to_string();
            self.save(&tasks);
        } else {
            panic!("Task with id {} not found", id);
        }
    }
}
