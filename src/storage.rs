use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDate};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Category {
    Review,
    Done,
    VoltouDeReview,
    Obstacle,
}

impl Category {
    pub fn label(&self) -> &str {
        match self {
            Category::Review => "Review",
            Category::Done => "Fiz",
            Category::VoltouDeReview => "Voltou de Review",
            Category::Obstacle => "Obstáculo",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Category::Review => "📋",
            Category::Done => "✅",
            Category::VoltouDeReview => "🔄",
            Category::Obstacle => "🚧",
        }
    }

    pub fn all() -> [Category; 4] {
        [
            Category::Review,
            Category::Done,
            Category::VoltouDeReview,
            Category::Obstacle,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub category: Category,
    pub description: String,
    pub created_at: DateTime<Local>,
}

impl Task {
    pub fn new(category: Category, description: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            category,
            description,
            created_at: Local::now(),
        }
    }
}

fn data_dir() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "omtl")
        .context("não foi possível determinar diretório de dados")?;
    let path = dirs.data_local_dir().join("tasks");
    fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn tasks_file(date: NaiveDate) -> Result<PathBuf> {
    Ok(data_dir()?.join(format!("{}.json", date.format("%Y-%m-%d"))))
}

pub fn load_tasks(date: NaiveDate) -> Result<Vec<Task>> {
    let path = tasks_file(date)?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn save_tasks(date: NaiveDate, tasks: &[Task]) -> Result<()> {
    let path = tasks_file(date)?;
    let content = serde_json::to_string_pretty(tasks)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn sent_log_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "omtl")
        .context("não foi possível determinar diretório de dados")?;
    let path = dirs.data_local_dir().to_path_buf();
    fs::create_dir_all(&path)?;
    Ok(path.join("sent.log"))
}

pub fn already_sent_today() -> bool {
    let Ok(path) = sent_log_path() else {
        return false;
    };
    if !path.exists() {
        return false;
    }
    let Ok(content) = fs::read_to_string(&path) else {
        return false;
    };
    let today = Local::now().date_naive().to_string();
    content.trim() == today
}

pub fn mark_sent_today() -> Result<()> {
    let path = sent_log_path()?;
    let today = Local::now().date_naive().to_string();
    fs::write(path, today)?;
    Ok(())
}
