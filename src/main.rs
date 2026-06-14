mod app;
mod config;
mod discord;
mod docker;
mod report;
mod scheduler;
mod storage;
mod tui;
mod ui;
mod whatsapp;
mod wpp_setup;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "omtl", about = "oh-my-tech-lead — rastreador de tarefas diárias")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Envia o relatório de hoje agora
    Send,
    /// Inicia o daemon de agendamento em background
    Daemon,
    /// Configura o app (bot token, user ID, horário)
    Setup,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => tui::run_tui()?,
        Some(Command::Send) => scheduler::send_now().await?,
        Some(Command::Daemon) => scheduler::run_daemon().await?,
        Some(Command::Setup) => run_setup().await?,
    }

    Ok(())
}

async fn run_setup() -> Result<()> {
    use std::io::{self, BufRead, Write};

    let stdin = io::stdin();
    let mut config = config::load_config()?;

    println!("=== oh-my-tech-lead — Setup ===");
    println!();
    println!("Para criar o bot Discord:");
    println!("  1. Acesse: discord.com/developers/applications");
    println!("  2. Crie um Application -> va em Bot -> copie o Token");
    println!("  3. Convide o bot para um servidor compartilhado com seu tech lead");
    println!("  4. No Discord: Configuracoes -> Avancado -> Ative Modo Desenvolvedor");
    println!("     Clique direito no tech lead -> Copiar ID do Usuario");
    println!();

    print!(
        "Bot Token [{}]: ",
        if config.discord.bot_token.is_empty() {
            "vazio".to_string()
        } else {
            "***".to_string()
        }
    );
    io::stdout().flush()?;
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    let trimmed = line.trim();
    if !trimmed.is_empty() {
        config.discord.bot_token = trimmed.to_string();
    }

    print!(
        "User ID do Tech Lead [{}]: ",
        if config.discord.tech_lead_user_id.is_empty() {
            "vazio".to_string()
        } else {
            config.discord.tech_lead_user_id.clone()
        }
    );
    io::stdout().flush()?;
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    let trimmed = line.trim();
    if !trimmed.is_empty() {
        config.discord.tech_lead_user_id = trimmed.to_string();
    }

    print!("Horario de envio (HH:MM) [{}]: ", config.schedule.send_time);
    io::stdout().flush()?;
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    let trimmed = line.trim();
    if !trimmed.is_empty() {
        config.schedule.send_time = trimmed.to_string();
    }

    // --- WhatsApp ---
    println!();
    print!("Ativar envio via WhatsApp? [s/N]: ");
    io::stdout().flush()?;
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    let wpp_enabled = matches!(line.trim().to_lowercase().as_str(), "s" | "sim");
    config.whatsapp.enabled = wpp_enabled;

    if wpp_enabled {
        println!();
        println!("Dica: use [W] no TUI para configurar o WhatsApp automaticamente via Docker + QR code.");
        println!("Ou rode o container WAHA manualmente:");
        println!("  docker run -d --name omtl-waha -p 127.0.0.1:3000:3000 -e WAHA_API_KEY=omtl-local-key devlikeapro/waha");
        println!();

        print!("WAHA URL [{}]: ", config.whatsapp.evolution_url);
        io::stdout().flush()?;
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        let trimmed = line.trim();
        if !trimmed.is_empty() { config.whatsapp.evolution_url = trimmed.to_string(); }

        print!("API Key [{}]: ", if config.whatsapp.api_key.is_empty() { "vazio" } else { "***" });
        io::stdout().flush()?;
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        let trimmed = line.trim();
        if !trimmed.is_empty() { config.whatsapp.api_key = trimmed.to_string(); }

        print!("Nome da instancia [{}]: ", config.whatsapp.instance);
        io::stdout().flush()?;
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        let trimmed = line.trim();
        if !trimmed.is_empty() { config.whatsapp.instance = trimmed.to_string(); }

        print!("Telefone do tech lead com DDI (ex: 5511999999999) [{}]: ",
            if config.whatsapp.tech_lead_phone.is_empty() { "vazio" } else { &config.whatsapp.tech_lead_phone });
        io::stdout().flush()?;
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        let trimmed = line.trim();
        if !trimmed.is_empty() { config.whatsapp.tech_lead_phone = trimmed.to_string(); }
    }

    config::save_config(&config)?;
    println!();
    println!("Configuracoes salvas em {:?}", config::config_path()?);

    install_systemd_service()?;

    Ok(())
}

fn install_systemd_service() -> Result<()> {
    use std::fs;
    use std::process::Command;

    let exe = std::env::current_exe()?;
    let service = format!(
        "[Unit]\nDescription=oh-my-tech-lead daemon\nAfter=network-online.target\n\n[Service]\nExecStart={} daemon\nRestart=always\nRestartSec=10\n\n[Install]\nWantedBy=default.target\n",
        exe.display()
    );

    let home = dirs_home()?;
    let systemd_dir = home.join(".config/systemd/user");
    fs::create_dir_all(&systemd_dir)?;
    let service_path = systemd_dir.join("omtl.service");
    fs::write(&service_path, &service)?;

    println!("Servico systemd instalado em {:?}", service_path);

    let status = Command::new("systemctl")
        .args(["--user", "enable", "--now", "omtl"])
        .status();

    match status {
        Ok(s) if s.success() => println!("Daemon iniciado via systemd!"),
        _ => {
            println!("Para iniciar o daemon manualmente:");
            println!("  systemctl --user enable --now omtl");
        }
    }

    Ok(())
}

fn dirs_home() -> Result<std::path::PathBuf> {
    directories::BaseDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("nao foi possivel determinar home dir"))
}
