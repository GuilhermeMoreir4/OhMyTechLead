use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::app::{WppPhase, WppState};
use crate::config::{load_config, save_config};
use crate::docker::{self, LOCAL_API_KEY, LOCAL_URL};
use crate::whatsapp::{WahaClient, render_qr};

macro_rules! set {
    ($state:expr, phase = $p:expr) => { $state.lock().unwrap().phase = $p; };
    ($state:expr, log = $msg:expr)  => { $state.lock().unwrap().log = $msg.to_string(); };
    ($state:expr, qr = $q:expr)     => { $state.lock().unwrap().qr_rendered = Some($q); };
}

fn cancelled(cancel: &AtomicBool) -> bool { cancel.load(Ordering::Relaxed) }

pub async fn run(state: Arc<Mutex<WppState>>, cancel: Arc<AtomicBool>) {
    if let Err(e) = run_inner(&state, &cancel).await {
        if !cancelled(&cancel) {
            let msg = e.to_string();
            set!(state, phase = WppPhase::Error(msg));
        }
    }
}

async fn run_inner(state: &Arc<Mutex<WppState>>, cancel: &Arc<AtomicBool>) -> anyhow::Result<()> {
    let config = load_config()?;

    // ── 1. Docker ────────────────────────────────────────────────────────────
    set!(state, phase = WppPhase::CheckingDocker);
    set!(state, log   = "Verificando Docker...");

    if !docker::is_docker_available() {
        anyhow::bail!("Docker não encontrado. Instale o Docker e tente novamente.");
    }
    if cancelled(cancel) { return Ok(()); }

    set!(state, phase = WppPhase::StartingContainer);
    set!(state, log   = "Iniciando waha (container único, sem banco de dados)...");
    docker::ensure_running()?;
    if cancelled(cancel) { return Ok(()); }

    // ── 2. Aguardar API ──────────────────────────────────────────────────────
    // Always use the local container URL and fixed API key, regardless of what
    // the user has configured (those settings are for external waha instances).
    set!(state, phase = WppPhase::WaitingApi);
    set!(state, log   = "Aguardando waha inicializar...");

    // WAHA Core (free) only supports the "default" session name.
    let session = "default".to_string();

    let client = WahaClient::new(LOCAL_URL, &session, LOCAL_API_KEY);
    client.wait_ready(60).await?;
    if cancelled(cancel) { return Ok(()); }

    // ── 3. Sessão ────────────────────────────────────────────────────────────
    set!(state, phase = WppPhase::CheckingInstance);
    set!(state, log   = "Verificando sessão WhatsApp...");

    if !client.session_exists().await {
        set!(state, phase = WppPhase::CreatingInstance);
        set!(state, log   = "Criando sessão...");
        client.create_session().await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    if cancelled(cancel) { return Ok(()); }

    // Se já estiver conectado, pula direto para o final
    if let Ok(status) = client.connection_state().await {
        if status == "WORKING" {
            return finish_connected(state, cancel, &config.whatsapp.tech_lead_phone, &session).await;
        }
        // Sessão existe mas está parada — inicia
        if status == "STOPPED" {
            set!(state, log = "Iniciando sessão existente...");
            client.start_session().await?;
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    // ── 4. Loop QR code ──────────────────────────────────────────────────────
    set!(state, phase = WppPhase::ShowingQr);
    let mut browser_opened = false;

    loop {
        if cancelled(cancel) { return Ok(()); }

        match client.get_qr_code().await {
            Ok(qr_data) => {
                if let Ok(rendered) = render_qr(&qr_data) {
                    set!(state, qr  = rendered);
                    set!(state, log = "Escaneie abaixo ou abra http://127.0.0.1:3000 no browser");
                }
                // Download PNG and open with default viewer (avoids auth header requirement)
                if !browser_opened {
                    browser_opened = true;
                    let png_url = format!("{}/api/{}/auth/qr", LOCAL_URL, session);
                    let key = LOCAL_API_KEY.to_string();
                    tokio::task::spawn(async move {
                        if let Ok(resp) = reqwest::Client::new()
                            .get(&png_url)
                            .header("X-Api-Key", &key)
                            .send().await
                        {
                            if let Ok(bytes) = resp.bytes().await {
                                let path = "/tmp/omtl-qr.png";
                                if std::fs::write(path, &bytes).is_ok() {
                                    // Pass DISPLAY so xdg-open can reach the graphical environment
                                    let display = std::env::var("DISPLAY").unwrap_or(":0".to_string());
                                    let _ = std::process::Command::new("xdg-open")
                                        .arg(path)
                                        .env("DISPLAY", display)
                                        .spawn();
                                }
                            }
                        }
                    });
                }
            }
            Err(_) => {
                set!(state, log = "Aguardando QR code ficar disponível...");
            }
        }

        // Verifica conexão a cada 5s, por até 25s (depois o QR expira e renovamos)
        for _ in 0..5 {
            tokio::time::sleep(Duration::from_secs(5)).await;
            if cancelled(cancel) { return Ok(()); }

            if let Ok(status) = client.connection_state().await {
                if status == "WORKING" {
                    return finish_connected(state, cancel, &config.whatsapp.tech_lead_phone, &session).await;
                }
            }
        }

        set!(state, log = "QR expirado, gerando novo...");
    }
}

async fn finish_connected(
    state: &Arc<Mutex<WppState>>,
    cancel: &Arc<AtomicBool>,
    saved_phone: &str,
    session: &str,
) -> anyhow::Result<()> {
    if cancelled(cancel) { return Ok(()); }

    // Save the local URL and API key to config so scheduler can use them
    let mut config = load_config()?;
    config.whatsapp.enabled = true;
    config.whatsapp.evolution_url = LOCAL_URL.to_string();
    config.whatsapp.api_key = LOCAL_API_KEY.to_string();
    config.whatsapp.instance = session.to_string();
    save_config(&config)?;

    set!(state, phase = WppPhase::Connected);
    set!(state, log   = "WhatsApp conectado!");

    if saved_phone.is_empty() {
        tokio::time::sleep(Duration::from_millis(600)).await;
        set!(state, phase = WppPhase::AskPhone);
        set!(state, log   = "Digite o número do tech lead com DDI (ex: 5511999999999)");
    } else {
        set!(state, phase = WppPhase::Done);
        set!(state, log   = "Tudo pronto! WhatsApp configurado.");
    }

    Ok(())
}
