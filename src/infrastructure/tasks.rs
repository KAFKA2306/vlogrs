use crate::domain::task::Task;
use crate::domain::TaskRepository as TaskRepositoryTrait;
use anyhow::{Context, Result};
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

    pub fn ensure_file(&self) -> Result<()> {
        if !self.path.exists() {
            fs::write(&self.path, "[]").context("Failed to create initial tasks file")?;
        }
        Ok(())
    }

    fn save(&self, tasks: &[Task]) -> Result<()> {
        let content = serde_json::to_string_pretty(tasks).context("Failed to serialize tasks")?;
        let tmp_path = self.path.with_extension("tmp");
        fs::write(&tmp_path, content).context("Failed to write temporary tasks file")?;
        fs::rename(&tmp_path, &self.path).context("Failed to rename tasks file")?;
        Ok(())
    }
}

impl TaskRepositoryTrait for TaskRepository {
    fn add(&self, task_type: &str, file_paths: Vec<String>) -> Result<Task> {
        let mut tasks = self.load()?;
        let task = Task {
            id: Uuid::now_v7().to_string(),
            created_at: Utc::now(),
            status: "pending".to_string(),
            task_type: task_type.to_string(),
            file_paths,
        };
        tasks.push(task.clone());
        self.save(&tasks)?;
        Ok(task)
    }

    fn load(&self) -> Result<Vec<Task>> {
        self.ensure_file()?;
        let content = fs::read_to_string(&self.path).context("Failed to read tasks file")?;
        serde_json::from_str(&content).context("Failed to parse tasks file")
    }

    fn update_status(&self, id: &str, status: &str) -> Result<()> {
        let mut tasks = self.load()?;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
            task.status = status.to_string();
            self.save(&tasks)
        } else {
            anyhow::bail!("Task with id {} not found", id)
        }
    }
}
