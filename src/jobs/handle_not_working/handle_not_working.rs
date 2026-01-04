use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use reqwest::Url;

use crate::{
    config::Config,
    debug, info,
    jobs::{
        enums::strike_type::StrikeType,
        handle_not_working::{action_taker::ActionTaker, notifier::Notifier, receiver::Receiver, striker::Striker},
        utils::{discord_webhook_utils::DiscordWebhookUtils, strike_utils::StrikeUtils},
    },
    logger::enums::category::Category,
    torrent_clients::{
        models::{torrent::Torrent, tracker::Tracker},
        torrent_manager::TorrentManager,
    },
};

pub struct HandleNotWorking {
    torrent_manager: Arc<TorrentManager>,
    config: Config,
}

impl HandleNotWorking {
    pub fn new(torrent_manager: Arc<TorrentManager>, config: Config) -> Self {
        Self { torrent_manager, config }
    }

    /**
     * Run
     */
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let discord_webhook_url: Option<Url> = match self.config.notification().discord_webhook_url().len() > 1 {
            true => Some(Url::parse(self.config.notification().discord_webhook_url()).context("Failed to parse discord_webhook_url")?),
            false => None,
        };
        let mut discord_webhook_utils = DiscordWebhookUtils::new(discord_webhook_url);

        // Login
        self.torrent_manager.login().await.context("Failed to login to torrent client")?;

        // Get torrents from torrent client
        debug!(Category::HandleNotWorking, "Getting torrents...");
        let torrents = self.torrent_manager.get_all_torrents().await.context("Failed to get all torrents")?;
        debug!(Category::HandleNotWorking, "Received {} torrents", torrents.len());

        // Get torrent trackers
        debug!(Category::HandleNotWorking, "Getting torrent trackers...");
        let torrent_trackers: HashMap<String, Vec<Tracker>> = Receiver::get_torrent_trackers(self.torrent_manager.clone(), &torrents, &self.config).await?;
        debug!(Category::HandleNotWorking, "Received torrent trackers");

        // Get torrents from torrent client with criteria
        debug!(Category::HandleNotWorking, "Checking torrents for criteria...");
        let torrents_criteria: HashMap<String, (Torrent, bool)> = Receiver::get_torrents_criteria(&torrents, &torrent_trackers, &self.config).await?;
        debug!(Category::HandleNotWorking, "Done checking torrents for criteria");

        info!(
            Category::HandleNotWorking,
            "{} torrents meet criteria",
            torrents_criteria.values().filter(|(_, is_criteria_met)| *is_criteria_met).count(),
        );

        // Striking
        debug!(Category::HandleNotWorking, "Striking torrents...");
        let mut strike_utils = StrikeUtils::new()?;
        let limit_reached_torrents = Striker::strike_torrents(&mut strike_utils, &torrents_criteria, &self.config)?;
        debug!(Category::HandleNotWorking, "Done striking torrents");

        info!(Category::HandleNotWorking, "{} torrents that meet criteria have reached their strike limits", limit_reached_torrents.len(),);

        // Go through torrents
        for torrent in &limit_reached_torrents {
            // Log
            info!(Category::HandleNotWorking, "Torrent not working: {}", torrent.name());

            // Notification
            if *self.config.notification().on_job_action() {
                let trackers = match torrent_trackers.get(torrent.hash()) {
                    Some(trackers) => trackers,
                    None => &Vec::new(),
                };
                Notifier::send_notification(&mut discord_webhook_utils, torrent, trackers, &self.config).await.context("Failed to send notification")?;
            }

            // Take action
            ActionTaker::take_action(self.torrent_manager.clone(), &torrents_criteria, torrent, &self.config).await?;
        }

        // Clean db
        debug!(Category::HandleNotWorking, "Cleaning db...");
        self.clean_db(&mut strike_utils, &torrents_criteria, &limit_reached_torrents)?;
        debug!(Category::HandleNotWorking, "Cleaned db");

        // Logout
        self.torrent_manager.logout().await.context("Failed to logout of torrent client")?;

        Ok(())
    }

    /**
     * Clean db
     */
    fn clean_db(&self, strike_utils: &mut StrikeUtils, torrents_criteria: &HashMap<String, (Torrent, bool)>, limit_reached_torrents: &[Torrent]) -> Result<(), anyhow::Error> {
        let mut hashes_to_remove: Vec<String> = Vec::new();

        // Torrents that reached limit and were handled
        let limit_reached_torrent_hashes: Vec<String> = limit_reached_torrents.iter().map(|torrent| torrent.hash().to_string()).collect();
        hashes_to_remove.extend(limit_reached_torrent_hashes);

        let strike_records = strike_utils.get_strikes(&StrikeType::HandleNotWorking, None).context("Failed to get all strikes for HandleNotWorking")?;
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

        debug!(Category::HandleNotWorking, "Deleting {} hashes", hashes_to_remove.len());

        strike_utils.delete(StrikeType::HandleNotWorking, hashes_to_remove).context("Failed to delete hashes")?;

        Ok(())
    }
}
