use anyhow::{bail, Context, Result};
use reqwest::{Client, RequestBuilder};
use serde_json::{json, Value};
use std::time::Duration;

pub struct WahaClient {
    client: Client,
    base_url: String,
    session: String,
    api_key: Option<String>,
}

impl WahaClient {
    pub fn new(base_url: &str, session: &str, api_key: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(8))
                .build()
                .unwrap_or_default(),
            base_url: base_url.trim_end_matches('/').to_string(),
            session: session.to_string(),
            api_key: if api_key.is_empty() { None } else { Some(api_key.to_string()) },
        }
    }

    fn get(&self, url: impl AsRef<str>) -> RequestBuilder {
        let req = self.client.get(url.as_ref());
        match &self.api_key {
            Some(k) => req.header("X-Api-Key", k),
            None => req,
        }
    }

    fn post(&self, url: impl AsRef<str>) -> RequestBuilder {
        let req = self.client.post(url.as_ref());
        match &self.api_key {
            Some(k) => req.header("X-Api-Key", k),
            None => req,
        }
    }

    pub async fn wait_ready(&self, max_secs: u64) -> Result<()> {
        let mut last_err = String::from("sem resposta");
        for _ in 0..max_secs {
            match self.get(format!("{}/api/sessions", self.base_url)).send().await {
                Ok(r) if r.status().as_u16() < 500 => return Ok(()),
                Ok(r) => last_err = format!("HTTP {}", r.status().as_u16()),
                Err(e) => last_err = e.to_string(),
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        bail!("waha não respondeu após {}s (último erro: {})", max_secs, last_err)
    }

    pub async fn session_exists(&self) -> bool {
        self.get(format!("{}/api/sessions/{}", self.base_url, self.session))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub async fn create_session(&self) -> Result<()> {
        let resp = self
            .post(format!("{}/api/sessions", self.base_url))
            .json(&json!({ "name": self.session }))
            .send()
            .await
            .context("falha ao criar sessão")?;

        if !resp.status().is_success() {
            let body: Value = resp.json().await.unwrap_or(json!({}));
            let msg = body["message"].as_str().unwrap_or("erro desconhecido");
            bail!("waha: {}", msg);
        }
        Ok(())
    }

    /// Returns the session status: "WORKING", "SCAN_QR_CODE", "STOPPED", etc.
    pub async fn connection_state(&self) -> Result<String> {
        let resp: Value = self
            .get(format!("{}/api/sessions/{}", self.base_url, self.session))
            .send()
            .await
            .context("falha ao checar estado da sessão")?
            .json()
            .await?;

        resp["status"]
            .as_str()
            .map(|s| s.to_string())
            .context("campo 'status' ausente na resposta")
    }

    pub async fn start_session(&self) -> Result<()> {
        let resp = self
            .post(format!("{}/api/sessions/{}/start", self.base_url, self.session))
            .send()
            .await
            .context("falha ao iniciar sessão")?;

        if !resp.status().is_success() {
            let body: Value = resp.json().await.unwrap_or(json!({}));
            let msg = body["message"].as_str().unwrap_or("erro desconhecido");
            anyhow::bail!("waha start: {}", msg);
        }
        Ok(())
    }

    /// Returns the raw QR code string (for terminal rendering).
    pub async fn get_qr_code(&self) -> Result<String> {
        let resp: Value = self
            .get(format!("{}/api/{}/auth/qr?format=raw", self.base_url, self.session))
            .send()
            .await
            .context("falha ao obter QR code")?
            .json()
            .await?;

        resp["value"]
            .as_str()
            .map(|s| s.to_string())
            .context("QR code ainda não disponível")
    }

    pub async fn send_text(&self, phone: &str, text: &str) -> Result<()> {
        let chat_id = if phone.contains('@') {
            phone.to_string()
        } else {
            // Strip everything except digits — WAHA expects "5511999999999@c.us"
            let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
            format!("{}@c.us", digits)
        };

        let resp = self
            .post(format!("{}/api/sendText", self.base_url))
            .json(&json!({
                "chatId": chat_id,
                "text": text,
                "session": self.session
            }))
            .send()
            .await
            .context("falha ao enviar mensagem via waha")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body: Value = resp.json().await.unwrap_or(json!({}));
            let msg = body["message"].as_str().unwrap_or("erro desconhecido");
            bail!("waha {}: {}", status.as_u16(), msg);
        }

        Ok(())
    }
}

/// Renders a QR code data string into unicode block characters for terminal display.
pub fn render_qr(data: &str) -> Result<String> {
    use qrcode::{EcLevel, QrCode};
    use qrcode::render::unicode;

    // EcLevel::L → menor versão de QR → módulos maiores → mais fácil escanear
    let code = QrCode::with_error_correction_level(data.as_bytes(), EcLevel::L)
        .context("falha ao gerar QR code")?;

    // Dense1x2 compensa o ratio 2:1 (altura:largura) dos caracteres do terminal.
    // module_dimensions(1, 1) = 1 char wide, half-block (2 módulos por linha).
    Ok(code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Dark)
        .light_color(unicode::Dense1x2::Light)
        .quiet_zone(true)
        .build())
}
