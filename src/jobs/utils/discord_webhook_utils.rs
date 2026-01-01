use std::time::Duration;

use chrono::Local;
use reqwest::{
    Client, StatusCode, Url,
    header::{HeaderMap, RETRY_AFTER},
};
use serde::Serialize;
use serde_json::{Value, json};
use tokio::time::sleep;

use crate::logger::{enums::category::Category, logger::Logger};

#[derive(Serialize)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

pub struct DiscordWebhookUtils {
    discord_webhook_url: Option<Url>,
    client: Client,
    request_history: Vec<(i64, bool)>, // timestamp (seconds), is_success
}

impl DiscordWebhookUtils {
    pub fn new(discord_webhook_url: Option<Url>) -> Self {
        Self {
            discord_webhook_url,
            client: Client::new(),
            request_history: Vec::new(),
        }
    }

    pub fn is_notifications_enabled(&self) -> bool {
        return self.discord_webhook_url.is_some();
    }

    async fn get_retry_after_millis(&mut self, headers: &HeaderMap) -> u64 {
        let timestamp_secs_now = Local::now().timestamp();
        let fail_count_in_last_10_secs = self.request_history.iter().filter(|(timestamp_secs, is_success)| (timestamp_secs_now - 10 <= *timestamp_secs) && !*is_success).count();
        if fail_count_in_last_10_secs >= 4 {
            Logger::warn(Category::DiscordNotifier, "4 or more requests to discord failed in the last 10 seconds, waiting 60 seconds to cooldown...");
            self.request_history.clear();
            sleep(Duration::from_secs(60)).await;
            Logger::info(Category::DiscordNotifier, "Cooled down, continuing...");
            return 0;
        }

        // Try get header_value
        match headers.get(RETRY_AFTER) {
            Some(header_value) => {
                // Try get string from header_value
                match header_value.to_str() {
                    Ok(str) => {
                        // Try to parse string to f64
                        match str.parse::<u64>() {
                            Ok(retry_after_millis) => {
                                retry_after_millis + 500 // +500 just to be safe, discord sometimes sends too low numbers and then directly sends 429 again
                            }
                            Err(e) => {
                                Logger::warn(Category::DiscordNotifier, format!("Failed to parse RETRY_AFTER header value to f64, using default 3000ms: {:#}", e).as_str());
                                3000
                            }
                        }
                    }
                    Err(e) => {
                        Logger::warn(Category::DiscordNotifier, format!("Failed to get string from RETRY_AFTER header, using default 3000ms: {:#}", e).as_str());
                        3000
                    }
                }
            }
            None => {
                Logger::warn(Category::DiscordNotifier, format!("Failed to get RETRY_AFTER from headers, using default 3000ms").as_str());
                3000
            }
        }
    }

    async fn make_request(&mut self, payload: &Value) -> Result<(), anyhow::Error> {
        let discord_webhook_url = match self.discord_webhook_url.clone() {
            Some(discord_webhook_url) => discord_webhook_url,
            None => return Ok(()),
        };
        loop {
            match self.client.post(discord_webhook_url.clone()).json(payload).send().await {
                // Got a response
                Ok(response) => {
                    // Success
                    if response.status().is_success() {
                        self.request_history.push((Local::now().timestamp(), true));
                        break;
                    }
                    // Error: 429
                    else if response.status() == StatusCode::TOO_MANY_REQUESTS {
                        self.request_history.push((Local::now().timestamp(), false));
                        let retry_after_millis = self.get_retry_after_millis(&response.headers()).await;
                        if retry_after_millis > 0 {
                            Logger::warn(
                                Category::DiscordNotifier,
                                format!("Received status code 429 (too many requests) from discord, waiting {:.2} seconds", (retry_after_millis as f64 / 1000.0)).as_str(),
                            );
                            sleep(Duration::from_millis(retry_after_millis)).await;
                        }
                    }
                    // Error: Unexpected
                    else {
                        self.request_history.push((Local::now().timestamp(), false));
                        anyhow::bail!(
                            "Sending discord notification failed with status code {}: {}",
                            response.status(),
                            response.text().await.unwrap_or_default(),
                        );
                    }
                }
                // Request itself failed
                Err(e) => {
                    self.request_history.push((Local::now().timestamp(), false));
                    anyhow::bail!("Sending discord notification failed: {:#}", e);
                }
            }
        }
        Ok(())
    }

    pub async fn send_webhook_embed(&mut self, title: &str, description: &str, fields: Vec<EmbedField>) -> Result<(), anyhow::Error> {
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
