use std::{fs, path::Path};

use anyhow::Context;
use chrono::{DateTime, Local};

use crate::{
    config::Config,
    jobs::utils::discord_webhook_utils::{DiscordWebhookUtils, EmbedField},
    logger::{enums::category::Category, logger::Logger},
};

pub struct Notifier;

impl Notifier {
    /**
     * Send notification
     */
    pub async fn send_notification(discord_webhook_utils: &mut DiscordWebhookUtils, path_str: &str, path: &Path, config: &Config) -> Result<(), anyhow::Error> {
        if !discord_webhook_utils.is_notifications_enabled() {
            return Ok(());
        }

        let metadata = fs::metadata(path).context("Failed to get file metadata")?;
        let file_size_gb_string = format!("{:.2}GB", (metadata.len() / 1000 / 1000) as f32 / 1000.0);
        let modified_time = metadata.modified().context("Failed to get file modified SystemTime")?;

        let modified_time: DateTime<Local> = modified_time.into();
        let modified_time: String = modified_time.format("%Y-%m-%d %H:%M:%S").to_string();

        let description = if path.is_file() {
            "Found orphaned **file**"
        } else if path.is_dir() {
            "Found orphaned **folder**"
        } else {
            Logger::warn(Category::HandleOrphaned, format!("Path is not file or folder: {}", path.display()).as_str());
            "Found orphaned path which isn't file or folder?"
        };

        let fields: Vec<EmbedField> = vec![
            EmbedField {
                name: String::from("Action"),
                value: config.jobs().handle_orphaned().action().to_string(),
                inline: false,
            },
            EmbedField {
                name: String::from("Size"),
                value: file_size_gb_string,
                inline: false,
            },
            EmbedField {
                name: String::from("Last modifed"),
                value: modified_time,
                inline: false,
            },
        ];

        discord_webhook_utils.send_webhook_embed(path_str, description, fields).await
    }
}
