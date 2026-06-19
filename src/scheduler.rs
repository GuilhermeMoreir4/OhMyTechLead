use anyhow::Result;
use chrono::{Datelike, Local, Timelike, Weekday};
use std::time::Duration;

use crate::config::{load_config, Config};
use crate::discord::DiscordClient;
use crate::report::generate_report;
use crate::storage::{
    already_sent_for, find_active_date, load_tasks, mark_sent_for, Category,
};
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

fn is_local_url(url: &str) -> bool {
    url.contains("127.0.0.1") || url.contains("localhost")
}

fn build_categories(config: &Config) -> Vec<Category> {
    let mut cats: Vec<Category> = Category::all().to_vec();
    for cc in &config.custom_categories {
        cats.push(Category::Custom {
            key: cc.key.clone(),
            name: cc.name.clone(),
            icon: cc.icon.clone(),
        });
    }
    cats
}

pub async fn run_daemon() -> Result<()> {
    println!("omtl daemon iniciado. Aguardando horário programado...");

    loop {
        let config = load_config()?;
        let now = Local::now();

        if is_weekday(now.weekday()) {
            if let Some((h, m)) = parse_send_time(&config.schedule.send_time) {
                if now.hour() == h && now.minute() == m {
                    let active_date = find_active_date();
                    if !already_sent_for(active_date) {
                        println!("[{}] Enviando relatório para {}...", now.format("%Y-%m-%d %H:%M"), active_date);
                        match send_now_internal(&config, active_date).await {
                            Ok(()) => {
                                println!("Relatório enviado com sucesso.");
                                mark_sent_for(active_date)?;
                            }
                            Err(e) => eprintln!("Erro ao enviar relatório: {e}"),
                        }
                    }
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

async fn send_now_internal(config: &Config, date: chrono::NaiveDate) -> Result<()> {
    let tasks = load_tasks(date)?;
    let categories = build_categories(config);
    let report = generate_report(date, &tasks, &categories);

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
        // Auto-start local container if needed
        if is_local_url(&config.whatsapp.evolution_url) {
            if let Err(e) = crate::docker::ensure_accessible() {
                errors.push(format!("Docker: {e}"));
            } else {
                let wpp = WahaClient::new(
                    &config.whatsapp.evolution_url,
                    &config.whatsapp.instance,
                    &config.whatsapp.api_key,
                );
                if let Err(e) = wpp.wait_ready(30).await {
                    errors.push(format!("WhatsApp (aguardar container): {e}"));
                } else if let Err(e) = wpp.send_text(&config.whatsapp.tech_lead_phone, &report).await {
                    errors.push(format!("WhatsApp: {e}"));
                }
            }
        } else {
            let wpp = WahaClient::new(
                &config.whatsapp.evolution_url,
                &config.whatsapp.instance,
                &config.whatsapp.api_key,
            );
            if let Err(e) = wpp.send_text(&config.whatsapp.tech_lead_phone, &report).await {
                errors.push(format!("WhatsApp: {e}"));
            }
        }
    }

    if !errors.is_empty() {
        anyhow::bail!(errors.join(" | "));
    }

    Ok(())
}

pub async fn send_now() -> Result<()> {
    let config = load_config()?;
    let active_date = find_active_date();
    send_now_internal(&config, active_date).await?;
    mark_sent_for(active_date)?;
    Ok(())
}
