use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::{json, Value};

pub struct DiscordClient {
    client: Client,
    token: String,
}

impl DiscordClient {
    pub fn new(token: &str) -> Self {
        // garante o prefixo "Bot " obrigatório na Authorization header
        let token = if token.starts_with("Bot ") {
            token.to_string()
        } else {
            format!("Bot {}", token)
        };
        Self {
            client: Client::new(),
            token,
        }
    }

    async fn get_dm_channel(&self, user_id: &str) -> Result<String> {
        let resp = self
            .client
            .post("https://discord.com/api/v10/users/@me/channels")
            .header("Authorization", &self.token)
            .header("Content-Type", "application/json")
            .json(&json!({ "recipient_id": user_id }))
            .send()
            .await
            .context("sem conexão com a API do Discord")?;

        let status = resp.status();
        let body: Value = resp.json().await.context("resposta não é JSON")?;

        if !status.is_success() {
            let msg = body["message"].as_str().unwrap_or("erro desconhecido");
            bail!("Discord API {}: {}", status.as_u16(), msg);
        }

        body["id"]
            .as_str()
            .map(|s| s.to_string())
            .context("campo 'id' ausente na resposta do Discord")
    }

    pub async fn send_dm(&self, user_id: &str, content: &str) -> Result<()> {
        let channel_id = self.get_dm_channel(user_id).await?;

        let resp = self
            .client
            .post(format!(
                "https://discord.com/api/v10/channels/{}/messages",
                channel_id
            ))
            .header("Authorization", &self.token)
            .header("Content-Type", "application/json")
            .json(&json!({ "content": content }))
            .send()
            .await
            .context("falha ao enviar mensagem Discord")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body: Value = resp.json().await.unwrap_or(json!({}));
            let msg = body["message"].as_str().unwrap_or("erro desconhecido");
            bail!("Discord {}: {}", status.as_u16(), msg);
        }

        Ok(())
    }
}
