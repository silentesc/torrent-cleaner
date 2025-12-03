use std::time::Duration;

use reqwest::{Client, Url};
use serde::Serialize;
use serde_json::{Value, json};
use tokio::time::sleep;

#[derive(Serialize)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

pub struct DiscordWebhookUtils {
    discord_webhook_url: Option<Url>,
    client: Client,
}

impl DiscordWebhookUtils {
    pub fn new(discord_webhook_url: Option<Url>) -> Self {
        Self {
            discord_webhook_url,
            client: Client::new(),
        }
    }

    pub fn is_notifications_enabled(&self) -> bool {
        return self.discord_webhook_url.is_some()
    }

    async fn make_request(&self, payload: &Value) -> Result<(), anyhow::Error> {
        if let Some(discord_webhook_url) = &self.discord_webhook_url {
            loop {
                match self.client.post(discord_webhook_url.clone()).json(payload).send().await {
                    Ok(response) => {
                        if response.status() == 429 {
                            let retry_after_seconds = match response.headers().get("retry_after") {
                                Some(header_value) => header_value.to_str().unwrap_or("1").parse().unwrap_or(1.0),
                                None => 1.0,
                            };
                            if retry_after_seconds > 0.0 {
                                sleep(Duration::from_secs_f64(retry_after_seconds)).await;
                            }
                        } else if response.status().is_success() {
                            break;
                        } else {
                            return Err(anyhow::anyhow!(
                                "Sending discord notification failed with status code {}: {}",
                                response.status(),
                                response.text().await.unwrap_or_default(),
                            ));
                        }
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("Sending discord notification failed: {:#}", e));
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn send_webhook_embed(&self, title: &str, description: &str, fields: Vec<EmbedField>) -> Result<(), anyhow::Error> {
        let now_iso = chrono::Utc::now().to_rfc3339();
        let payload = json!({
            "username": "Torrent Cleaner",
            "embeds": [
                {
                    "title": title,
                    "description": description,
                    "color": 0x697cff,
                    "fields": fields,
                    "timestamp": now_iso,
                }
            ]
        });
        self.make_request(&payload).await
    }
}
