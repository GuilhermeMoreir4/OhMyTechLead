use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCategoryConfig {
    pub key: String,
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub bot_token: String,
    #[serde(default)]
    pub tech_lead_user_id: String,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bot_token: String::new(),
            tech_lead_user_id: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsappConfig {
    pub enabled: bool,
    pub evolution_url: String,
    pub api_key: String,
    pub instance: String,
    pub tech_lead_phone: String,
}

impl Default for WhatsappConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            evolution_url: "http://localhost:3000".to_string(),
            api_key: String::new(),
            instance: "default".to_string(),
            tech_lead_phone: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub send_time: String,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            send_time: "09:00".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub discord: DiscordConfig,
    #[serde(default)]
    pub whatsapp: WhatsappConfig,
    #[serde(default)]
    pub schedule: ScheduleConfig,
    #[serde(default)]
    pub custom_categories: Vec<CustomCategoryConfig>,
}

pub fn config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "omtl")
        .context("não foi possível determinar diretório de config")?;
    let path = dirs.config_dir().to_path_buf();
    fs::create_dir_all(&path)?;
    Ok(path.join("config.toml"))
}

pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = fs::read_to_string(&path)?;
    Ok(toml::from_str(&content)?)
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = config_path()?;
    let content = toml::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}
