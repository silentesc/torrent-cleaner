use std::{collections::HashSet, path::Path, sync::Arc};

use anyhow::Context;
use reqwest::Url;

use crate::{
    config::Config,
    debug, info,
    jobs::{
        enums::strike_type::StrikeType,
        handle_orphaned::{action_taker::ActionTaker, notifier::Notifier, receiver::Receiver, striker::Striker},
        utils::strike_utils::StrikeUtils,
    },
    logger::enums::category::Category,
    torrent_clients::torrent_manager::TorrentManager, utils::discord_webhook_utils::DiscordWebhookUtils,
};

pub struct HandleOrphaned {
    torrent_manager: Arc<TorrentManager>,
    config: Config,
    torrents_path: String,
}

impl HandleOrphaned {
    pub fn new(torrent_manager: Arc<TorrentManager>, config: Config, torrents_path: String) -> Self {
        Self { torrent_manager, config, torrents_path }
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let discord_webhook_url: Option<Url> = match self.config.notification().discord_webhook_url().len() > 1 {
            true => Some(Url::parse(self.config.notification().discord_webhook_url()).context("Failed to parse discord_webhook_url")?),
            false => None,
        };
        let mut discord_webhook_utils = DiscordWebhookUtils::new(discord_webhook_url);

        // Login
        self.torrent_manager.login().await.context("Failed to login to torrent client")?;

        // Get torrent_paths
        let torrent_paths = Receiver::get_torrent_paths(self.torrent_manager.clone()).await?;

        // Get orphaned_path_strings
        let orphaned_path_strings = Receiver::get_orphaned_path_strings(&torrent_paths, &self.torrents_path, *self.config.jobs().handle_orphaned().protect_external_hardlinks()).await?;

        let mut strike_utils = StrikeUtils::new()?;

        // Strike orphaned paths
        debug!(Category::HandleOrphaned, "Striking orphaned paths...");
        let limit_reached_path_strings = Striker::strike_paths(&mut strike_utils, orphaned_path_strings.iter().cloned().collect(), &self.config)?;
        debug!(Category::HandleOrphaned, "Done striking paths");

        info!(Category::HandleOrphaned, "{} paths have reached their strike limits", limit_reached_path_strings.len());

        // Go through paths
        for path_string in &limit_reached_path_strings {
            let path = Path::new(path_string.as_str());

            // Log
            info!(Category::HandleOrphaned, "Orphaned path: {}", path_string);

            // Notification
            if *self.config.notification().on_job_action() {
                Notifier::send_notification(&mut discord_webhook_utils, path_string.as_str(), path, &self.config)
                    .await
                    .context("Failed to send notification")?;
            }

            // Take action
            ActionTaker::take_action(path, &self.config)?;
        }

        // Clean db
        debug!(Category::HandleOrphaned, "Cleaning db...");
        self.clean_db(&mut strike_utils, &orphaned_path_strings, limit_reached_path_strings)?;
        debug!(Category::HandleOrphaned, "Cleaned db");

        // Logout
        self.torrent_manager.logout().await.context("Failed to logout of torrent client")?;

        Ok(())
    }

    /**
     * Clean db
     */
    fn clean_db(&self, strike_utils: &mut StrikeUtils, orphaned_path_strings: &HashSet<String>, limit_reached_path_strings: Vec<String>) -> Result<(), anyhow::Error> {
        let mut hashes_to_remove: Vec<String> = Vec::new();

        // Paths that reached limit and were handled from db
        hashes_to_remove.extend(limit_reached_path_strings);

        let strike_records = strike_utils.get_strikes(&StrikeType::HandleOrphaned, None).context("Failed to get all strikes for HandleOrphaned")?;
        for strike_record in strike_records {
            // Paths that not orphaned anymore
            if !orphaned_path_strings.contains(strike_record.hash()) {
                hashes_to_remove.push(strike_record.hash().to_string());
            }
        }

        debug!(Category::HandleOrphaned, "Deleting {} paths from strike db", hashes_to_remove.len());

        strike_utils.delete(StrikeType::HandleOrphaned, hashes_to_remove).context("Failed to delete paths from strike db")?;

        Ok(())
    }
}
