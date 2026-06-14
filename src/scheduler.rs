use anyhow::Result;
use chrono::{Datelike, Local, Timelike, Weekday};
use std::time::Duration;

use crate::config::{load_config, Config};
use crate::discord::DiscordClient;
use crate::report::generate_report;
use crate::storage::{already_sent_today, load_tasks, mark_sent_today};
use crate::whatsapp::WahaClient;

fn is_weekday(weekday: Weekday) -> bool {
    !matches!(weekday, Weekday::Sat | Weekday::Sun)
}

fn parse_send_time(time_str: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let hour: u32 = parts[0].parse().ok()?;
    let minute: u32 = parts[1].parse().ok()?;
    Some((hour, minute))
}

pub async fn run_daemon() -> Result<()> {
    println!("omtl daemon iniciado. Aguardando horário programado...");

    loop {
        let config = load_config()?;
        let now = Local::now();

        if is_weekday(now.weekday()) {
            if let Some((h, m)) = parse_send_time(&config.schedule.send_time) {
                if now.hour() == h && now.minute() == m && !already_sent_today() {
                    println!("[{}] Enviando relatório...", now.format("%Y-%m-%d %H:%M"));
                    match send_now_internal(&config).await {
                        Ok(()) => {
                            println!("Relatório enviado com sucesso.");
                            mark_sent_today()?;
                        }
                        Err(e) => eprintln!("Erro ao enviar relatório: {e}"),
                    }
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

async fn send_now_internal(config: &Config) -> Result<()> {
    let today = Local::now().date_naive();
    let tasks = load_tasks(today)?;
    let report = generate_report(today, &tasks);

    let mut errors: Vec<String> = Vec::new();

    // Discord
    if config.discord.enabled && !config.discord.bot_token.is_empty() && !config.discord.tech_lead_user_id.is_empty() {
        let discord = DiscordClient::new(&config.discord.bot_token);
        if let Err(e) = discord.send_dm(&config.discord.tech_lead_user_id, &report).await {
            errors.push(format!("Discord: {e}"));
        }
    }

    // WhatsApp via waha
    if config.whatsapp.enabled && !config.whatsapp.tech_lead_phone.is_empty() {
        let wpp = WahaClient::new(&config.whatsapp.evolution_url, &config.whatsapp.instance, &config.whatsapp.api_key);
        if let Err(e) = wpp.send_text(&config.whatsapp.tech_lead_phone, &report).await {
            errors.push(format!("WhatsApp: {e}"));
        }
    }

    if !errors.is_empty() {
        anyhow::bail!(errors.join(" | "));
    }

    Ok(())
}

pub async fn send_now() -> Result<()> {
    let config = load_config()?;
    send_now_internal(&config).await?;
    mark_sent_today()?;
    Ok(())
}
