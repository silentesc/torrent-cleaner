use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use reqwest::Url;

use crate::{
    config::Config,
    jobs::{
        enums::strike_type::StrikeType,
        handle_forgotten::{action_taker::ActionTaker, notifier::Notifier, receiver::Receiver, striker::Striker},
        utils::{discord_webhook_utils::DiscordWebhookUtils, strike_utils::StrikeUtils},
    },
    logger::logger::Logger,
    torrent_clients::{models::torrent::Torrent, torrent_manager::TorrentManager},
};

pub struct HandleForgotten {
    torrent_manager: Arc<TorrentManager>,
    media_folder_path: String,
    config: Config,
}

impl HandleForgotten {
    pub fn new(torrent_manager: Arc<TorrentManager>, media_folder_path: String, config: Config) -> Self {
        Self {
            torrent_manager,
            media_folder_path,
            config,
        }
    }

    /**
     * Run
     */
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let discord_webhook_url: Option<Url> = match self.config.notification().discord_webhook_url().len() > 1 {
            true => Some(Url::parse(self.config.notification().discord_webhook_url()).context("[handle_forgotten] Failed to parse discord_webhook_url")?),
            false => None,
        };
        let mut discord_webhook_utils = DiscordWebhookUtils::new(discord_webhook_url);

        // Login
        self.torrent_manager.login().await.context("[handle_forgotten] Failed to login to torrent client")?;

        // Get torrents from torrent client with criteria
        let torrents_criteria: HashMap<String, (Torrent, bool)> = Receiver::get_torrents_criteria(self.torrent_manager.clone(), &self.config, &self.media_folder_path).await?;

        Logger::info(format!("[handle_forgotten] {} torrents meet criteria", torrents_criteria.values().filter(|(_, is_criteria_met)| *is_criteria_met).count()).as_str());

        // Striking
        Logger::debug("[handle_forgotten] Striking torrents...");
        let mut strike_utils = StrikeUtils::new()?;
        let limit_reached_torrents = Striker::strike_torrents(&mut strike_utils, &torrents_criteria, &self.config)?;
        Logger::debug("[handle_forgotten] Done striking torrents");

        Logger::info(format!("[handle_forgotten] {} torrents that meet criteria have reached their strike limits", limit_reached_torrents.len()).as_str());

        // Go through torrents
        for torrent in &limit_reached_torrents {
            // Log
            Logger::info(format!("[handle_forgotten] Torrent forgotten: {}", torrent.name()).as_str());

            // Notification
            Notifier::send_notification(&mut discord_webhook_utils, &torrent, &self.config)
                .await
                .context("[handle_forgotten] Failed to send notification")?;

            // Take action
            ActionTaker::take_action(self.torrent_manager.clone(), &torrents_criteria, &torrent, &self.config).await?;
        }

        // Clean db
        Logger::debug("[handle_forgotten] Cleaning db...");
        self.clean_db(&mut strike_utils, &torrents_criteria, &limit_reached_torrents)?;
        Logger::debug("[handle_forgotten] Cleaned db");

        // Logout
        self.torrent_manager.logout().await.context("[handle_forgotten] Failed to logout of torrent client")?;

        Ok(())
    }

    /**
     * Clean db
     */
    fn clean_db(&self, strike_utils: &mut StrikeUtils, torrents_criteria: &HashMap<String, (Torrent, bool)>, limit_reached_torrents: &Vec<Torrent>) -> Result<(), anyhow::Error> {
        let mut hashes_to_remove: Vec<String> = Vec::new();

        // Torrents that reached limit and were handled
        let limit_reached_torrent_hashes: Vec<String> = limit_reached_torrents.iter().map(|torrent| torrent.hash().to_string()).collect();
        hashes_to_remove.extend(limit_reached_torrent_hashes);

        let strike_records = strike_utils.get_strikes(&StrikeType::HandleForgotten, None).context("[handle_forgotten] Failed to get all strikes for HandleForgotten")?;
        for strike_record in strike_records {
            match torrents_criteria.get(strike_record.hash()) {
                // Check for stuff that doesn't meet criteria
                Some((_, is_criteria_met)) => {
                    if !*is_criteria_met {
                        hashes_to_remove.push(strike_record.hash().to_string());
                    }
                }
                // Check for stuff that doesn't exist in torrents anymore
                None => {
                    hashes_to_remove.push(strike_record.hash().to_string());
                }
            }
        }

        Logger::debug(format!("[handle_forgotten] Deleting {} hashes", hashes_to_remove.len()).as_str());

        strike_utils.delete(StrikeType::HandleForgotten, hashes_to_remove).context("[handle_forgotten] Failed to delete hashes")?;

        Ok(())
    }
}
