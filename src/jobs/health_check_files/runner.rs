use std::{os::unix::fs::MetadataExt, path::Path, sync::Arc};

use anyhow::Context;
use reqwest::Url;

use crate::{
    config::Config,
    debug, info,
    jobs::enums::action_type::ActionType,
    logger::enums::category::Category,
    torrent_clients::{models::torrent::Torrent, torrent_manager::TorrentManager},
    utils::discord_webhook_utils::DiscordWebhookUtils,
    warn,
};

pub struct HealthCheckFiles {
    torrent_manager: Arc<TorrentManager>,
    config: Config,
}

impl HealthCheckFiles {
    pub fn new(torrent_manager: Arc<TorrentManager>, config: Config) -> Self {
        HealthCheckFiles { torrent_manager, config }
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
        debug!(Category::HealthCheckFiles, "Getting torrents...");
        let torrents = self.torrent_manager.get_all_torrents().await.context("Failed to get all torrents")?;
        debug!(Category::HealthCheckFiles, "Received {} torrents", torrents.len());

        debug!(Category::HealthCheckFiles, "Running file check...");
        let file_issues = self.check_files(self.torrent_manager.clone(), &torrents).await.context("Error while checking files for health check")?;
        debug!(Category::HealthCheckFiles, "File check reported {} issues", file_issues.len());

        // Handle file issues
        let action_type = ActionType::from_str(self.config.jobs().health_check_files().action())?;
        for file_issue in file_issues {
            warn!(Category::HealthCheckFiles, "File check: {}", file_issue);

            if *self.config.notification().on_job_action() {
                discord_webhook_utils.send_webhook_embed("Health Check (Files)", &file_issue, Vec::new()).await?;
            }

            match action_type {
                ActionType::Test => info!(Category::HealthCheckFiles, "Action: test"),
                ActionType::Stop => warn!(Category::HealthCheckFiles, "Stop action not supported on health_check_files"),
                ActionType::Delete => warn!(Category::HealthCheckFiles, "Delete action not supported on health_check_files"),
            }
        }

        // Logout
        self.torrent_manager.logout().await.context("Failed to logout to torrent client")?;

        Ok(())
    }

    pub async fn check_files(&self, torrent_manager: Arc<TorrentManager>, torrents: &Vec<Torrent>) -> Result<Vec<String>, anyhow::Error> {
        let mut issues: Vec<String> = Vec::new();

        for torrent in torrents {
            if *torrent.completion_on() == -1 {
                debug!(Category::HealthCheckFiles, "Torrent not completed: ({}) {}", torrent.hash(), torrent.name());
                continue;
            }
            let torrent_files = torrent_manager.get_torrent_files(torrent.hash()).await.context("Getting torrent file failed")?;
            for torrent_file in torrent_files {
                let path_str = format!("{}/{}", torrent.save_path(), torrent_file.name());
                let path = Path::new(&path_str);

                // Check exist
                if !path.try_exists().context(format!("Failed to check if file exists: {}", path_str))? {
                    issues.push(format!("File for torrent ({}) {} does not exist: {}", torrent.name(), torrent.hash(), path_str));
                    return Ok(issues);
                }

                let metadata = Path::new(&path_str).metadata().context(format!("Failed getting metadata of {}", path_str))?;

                // Check size
                if *torrent_file.size() != metadata.size() {
                    issues.push(format!(
                        "Torrent file size ({} bytes) doesn't match actual file size ({} bytes) for torrent ({}) {}: {}",
                        torrent_file.size(),
                        metadata.size(),
                        torrent.name(),
                        torrent.hash(),
                        path_str
                    ));
                }

                // Check file is dir
                if metadata.file_type().is_dir() {
                    issues.push(format!("Torrent file is actually a directory for torrent {} ({}): {}", torrent.name(), torrent.hash(), path_str));
                }
            }
        }

        Ok(issues)
    }
}
