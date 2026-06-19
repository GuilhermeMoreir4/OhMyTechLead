use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Local, NaiveDate, Weekday};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Category {
    Review,
    Done,
    VoltouDeReview,
    Obstacle,
    Custom { key: String, name: String, icon: String },
}

impl Category {
    pub fn label(&self) -> &str {
        match self {
            Category::Review => "Review",
            Category::Done => "Fiz",
            Category::VoltouDeReview => "Voltou de Review",
            Category::Obstacle => "Obstáculo",
            Category::Custom { name, .. } => name.as_str(),
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Category::Review => "📋",
            Category::Done => "✅",
            Category::VoltouDeReview => "🔄",
            Category::Obstacle => "🚧",
            Category::Custom { icon, .. } => icon.as_str(),
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

/// Returns true if `date` appears in sent.log (supports both old single-line and new multi-line format).
pub fn already_sent_for(date: NaiveDate) -> bool {
    let Ok(path) = sent_log_path() else { return false; };
    if !path.exists() { return false; }
    let Ok(content) = fs::read_to_string(&path) else { return false; };
    let date_str = date.to_string();
    content.lines().any(|line| line.trim() == date_str)
}

/// Appends `date` to sent.log (one date per line).
pub fn mark_sent_for(date: NaiveDate) -> Result<()> {
    let path = sent_log_path()?;
    let mut file = fs::OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", date)?;
    Ok(())
}

/// Returns true if the tasks file for `date` exists and contains at least one task.
pub fn has_tasks_for(date: NaiveDate) -> bool {
    let path = match tasks_file(date) {
        Ok(p) => p,
        Err(_) => return false,
    };
    if !path.exists() { return false; }
    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
        .and_then(|v| v.as_array().map(|a| !a.is_empty()))
        .unwrap_or(false)
}

/// Returns the most recent business day that has unsent tasks, or today if none found.
/// On Monday, this naturally returns Friday if Friday had unsent tasks.
pub fn find_active_date() -> NaiveDate {
    let today = Local::now().date_naive();
    let mut check = today;

    for _ in 0..10 {
        if !matches!(check.weekday(), Weekday::Sat | Weekday::Sun) {
            if has_tasks_for(check) && !already_sent_for(check) {
                return check;
            }
        }
        match check.pred_opt() {
            Some(prev) => check = prev,
            None => break,
        }
    }

    today
}
