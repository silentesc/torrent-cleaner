use std::time::Duration;

use reqwest::{
    Client, StatusCode, Url,
    header::{HeaderMap, RETRY_AFTER},
};
use serde::Serialize;
use serde_json::{Value, json};
use tokio::time::sleep;

use crate::logger::logger::Logger;

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
        return self.discord_webhook_url.is_some();
    }

    fn get_retry_after_millis(&self, headers: &HeaderMap) -> u64 {
        // Try get header_value
        match headers.get(RETRY_AFTER) {
            Some(header_value) => {
                // Try get string from header_value
                match header_value.to_str() {
                    Ok(str) => {
                        // Try to parse string to f64
                        match str.parse::<u64>() {
                            Ok(retry_after_millis) => {
                                retry_after_millis + 1000 // +1000 just to be safe, discord sometimes sends too low numbers and then directly sends 429 again
                            }
                            Err(e) => {
                                Logger::warn(format!("Failed to parse RETRY_AFTER header value to f64, using default 3000ms: {:#}", e).as_str());
                                3000
                            }
                        }
                    }
                    Err(e) => {
                        Logger::warn(format!("Failed to get string from RETRY_AFTER header, using default 3000ms: {:#}", e).as_str());
                        3000
                    }
                }
            }
            None => {
                Logger::warn(format!("Failed to get RETRY_AFTER from headers, using default 3000ms").as_str());
                3000
            }
        }
    }

    async fn make_request(&self, payload: &Value) -> Result<(), anyhow::Error> {
        if let Some(discord_webhook_url) = &self.discord_webhook_url {
            loop {
                match self.client.post(discord_webhook_url.clone()).json(payload).send().await {
                    Ok(response) => {
                        if response.status() == StatusCode::TOO_MANY_REQUESTS {
                            let retry_after_millis = self.get_retry_after_millis(&response.headers());
                            if retry_after_millis > 0 {
                                Logger::warn(format!("Received status code 429 (too many requests) from discord, waiting {:.2} seconds", (retry_after_millis as f64 / 1000.0)).as_str());
                                sleep(Duration::from_millis(retry_after_millis)).await;
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
