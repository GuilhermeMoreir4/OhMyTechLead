use anyhow::{bail, Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const CONTAINER: &str = "omtl-waha";
const IMAGE: &str = "devlikeapro/waha";
const PORT: u16 = 3000;

/// Fixed API key used for the auto-managed local container.
/// Stored in the waha container via WAHA_API_KEY env var so it never changes.
pub const LOCAL_API_KEY: &str = "omtl-local-key";
pub const LOCAL_URL: &str = "http://127.0.0.1:3000";

fn sessions_dir() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "omtl")
        .context("não foi possível determinar diretório de dados")?;
    let path = dirs.data_local_dir().join("waha-sessions");
    fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn is_docker_available() -> bool {
    Command::new("docker")
        .args(["info", "--format", "{{.ID}}"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn container_exists() -> bool {
    Command::new("docker")
        .args(["ps", "-a", "--filter", &format!("name=^/{}$", CONTAINER), "--format", "{{.Names}}"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(CONTAINER))
        .unwrap_or(false)
}

fn is_running() -> bool {
    Command::new("docker")
        .args(["ps", "--filter", &format!("name=^/{}$", CONTAINER), "--format", "{{.Names}}"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(CONTAINER))
        .unwrap_or(false)
}

/// Ensures the container is accessible without recreating it if already running.
/// - Running → noop
/// - Stopped (exists but not running) → `docker start`
/// - Missing → create fresh (same as ensure_running)
pub fn ensure_accessible() -> Result<()> {
    if is_running() {
        return Ok(());
    }
    if container_exists() {
        let ok = Command::new("docker")
            .args(["start", CONTAINER])
            .status()?
            .success();
        if !ok {
            anyhow::bail!("falha ao iniciar container {}", CONTAINER);
        }
        return Ok(());
    }
    // Container does not exist — create it
    let sessions = sessions_dir()?;
    let port_bind = format!("127.0.0.1:{}:3000", PORT);
    let volume = format!("{}:/app/.sessions", sessions.display());
    let api_key_env = format!("WAHA_API_KEY={}", LOCAL_API_KEY);

    let ok = Command::new("docker")
        .args([
            "run", "-d",
            "--name", CONTAINER,
            "-p", &port_bind,
            "-v", &volume,
            "-e", &api_key_env,
            IMAGE,
        ])
        .status()?
        .success();

    if !ok {
        anyhow::bail!("falha ao criar container {}", CONTAINER);
    }

    Ok(())
}

fn stop_and_remove() {
    let _ = Command::new("docker").args(["stop", CONTAINER]).output();
    let _ = Command::new("docker").args(["rm", CONTAINER]).output();
}

/// Stops any existing omtl-waha container and creates a fresh one with a
/// known API key and explicit IPv4 binding. WhatsApp session data is preserved
/// because it lives in the host volume.
pub fn ensure_running() -> Result<()> {
    if container_exists() {
        stop_and_remove();
    }

    let sessions = sessions_dir()?;
    let port_bind = format!("127.0.0.1:{}:3000", PORT);
    let volume = format!("{}:/app/.sessions", sessions.display());
    let api_key_env = format!("WAHA_API_KEY={}", LOCAL_API_KEY);

    let ok = Command::new("docker")
        .args([
            "run", "-d",
            "--name", CONTAINER,
            "-p", &port_bind,
            "-v", &volume,
            "-e", &api_key_env,
            IMAGE,
        ])
        .status()?
        .success();

    if !ok {
        bail!("falha ao criar container {}", CONTAINER);
    }

    Ok(())
}
