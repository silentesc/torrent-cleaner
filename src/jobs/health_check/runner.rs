use std::sync::Arc;

use anyhow::Context;
use reqwest::Url;

use crate::{config::Config, debug, jobs::health_check::file_health::FileHealth, logger::enums::category::Category, torrent_clients::torrent_manager::TorrentManager, utils::discord_webhook_utils::DiscordWebhookUtils, warn};

pub struct HealthCheck {
    torrent_manager: Arc<TorrentManager>,
    config: Config,
}

impl HealthCheck {
    pub fn new(torrent_manager: Arc<TorrentManager>, config: Config) -> Self {
        HealthCheck { torrent_manager, config }
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let discord_webhook_url: Option<Url> = match self.config.notification().discord_webhook_url().len() > 1 {
            true => Some(Url::parse(self.config.notification().discord_webhook_url()).context("Failed to parse discord_webhook_url")?),
            false => None,
        };
        let mut discord_webhook_utils = DiscordWebhookUtils::new(discord_webhook_url);

        // Login
        self.torrent_manager.login().await.context("Failed to login to torrent client")?;

        // Get torrents from torrent client
        debug!(Category::HealthCheck, "Getting torrents...");
        let torrents = self.torrent_manager.get_all_torrents().await.context("Failed to get all torrents")?;
        debug!(Category::HealthCheck, "Received {} torrents", torrents.len());

        debug!(Category::HealthCheck, "Running file check...");
        let file_issues = FileHealth::check_files(self.torrent_manager.clone(), &torrents).await.context("Error while checking files for health check")?;
        debug!(Category::HealthCheck, "File check reported {} issues", file_issues.len());

        // Handle file issues
        for file_issue in file_issues {
            warn!(Category::HealthCheck, "File check: {}", file_issue);
            if *self.config.notification().on_job_action() {
                discord_webhook_utils.send_webhook_embed("Health Check (Files)", &file_issue, Vec::new()).await?;
            }
        }

        // Logout
        self.torrent_manager.logout().await.context("Failed to logout to torrent client")?;

        Ok(())
    }
}
