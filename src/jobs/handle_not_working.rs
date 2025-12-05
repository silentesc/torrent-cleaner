use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use chrono::{Local, TimeZone};
use reqwest::Url;

use crate::{
    config::Config,
    jobs::{
        enums::{action_type::ActionType, strike_type::StrikeType},
        utils::{
            discord_webhook_utils::{DiscordWebhookUtils, EmbedField},
            strike_utils::StrikeUtils,
        },
    },
    logger::logger::Logger,
    torrent_clients::{
        enums::{torrent_state::TorrentState, tracker_status::TrackerStatus},
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
            true => Some(Url::parse(self.config.notification().discord_webhook_url()).context("[handle_not_working] Failed to parse discord_webhook_url")?),
            false => None,
        };
        let discord_webhook_utils = DiscordWebhookUtils::new(discord_webhook_url);

        // Login
        self.torrent_manager.login().await.context("[handle_not_working] Failed to login to torrent client")?;

        // Get torrents from torrent client
        Logger::debug("[handle_not_working] Getting torrents...");
        let torrents = self.torrent_manager.get_all_torrents().await.context("[handle_not_working] Failed to get all torrents")?;
        Logger::debug(format!("[handle_not_working] Received {} torrents", torrents.len()).as_str());

        // Get trackers
        Logger::debug("[handle_not_working] Getting torrent trackers...");
        let mut torrent_trackers: HashMap<String, Vec<Tracker>> = HashMap::new();
        for torrent in torrents.clone() {
            let trackers = self
                .torrent_manager
                .get_torrent_trackers(torrent.hash())
                .await
                .context(format!("[handle_not_working] Failed to get trackers for torrent: ({}) {}", torrent.hash(), torrent.name()))?;
            torrent_trackers.insert(torrent.hash().to_string(), trackers);
        }
        Logger::debug("[handle_not_working] Received torrent trackers");

        // Check torrents for criteria
        Logger::debug("[handle_not_working] Checking torrents for criteria...");
        let mut torrents_criteria: HashMap<String, (Torrent, bool)> = HashMap::new();
        for torrent in torrents.clone() {
            if let Some(trackers) = torrent_trackers.get(torrent.hash()) {
                torrents_criteria.insert(torrent.hash().to_string(), (torrent.clone(), self.is_criteria_met(&torrent, trackers).await));
            } else {
                Logger::warn(format!("[handle_not_working] Cannot get tracker for torrent: ({}) {}", torrent.hash(), torrent.name()).as_str());
            }
        }
        Logger::debug("[handle_not_working] Done checking torrents for criteria");

        // Striking
        Logger::debug("[handle_not_working] Striking torrents...");
        let mut strike_utils = StrikeUtils::new()?;
        let limit_reached_torrents = self.strike_torrents(&mut strike_utils, &torrents_criteria)?;
        Logger::debug(format!("[handle_not_working] Done striking, {} torrents reached their limit. Action will be taken now", limit_reached_torrents.len()).as_str());

        // Go through torrents
        for torrent in limit_reached_torrents.clone() {
            // Log
            Logger::info(format!("[handle_not_working] Torrent not working: {}", torrent.name()).as_str());

            // Notification
            let trackers = match torrent_trackers.get(torrent.hash()) {
                Some(trackers) => trackers,
                None => &Vec::new(),
            };
            self.send_notification(&discord_webhook_utils, &torrent, &trackers).await.context("[handle_not_working] Failed to send notification")?;

            // Take action
            self.take_action(&torrents_criteria, &torrent).await?;
        }

        // Remove torrents that reached limit and were handled from db
        let limit_reached_torrent_hashes: Vec<String> = limit_reached_torrents.iter().map(|torrent| torrent.hash().to_string()).collect();
        strike_utils.delete(StrikeType::HandleNotWorking, limit_reached_torrent_hashes)?;

        // Clean db
        Logger::debug("[handle_not_working] Cleaning db...");
        self.clean_db(&mut strike_utils, &torrents_criteria)?;
        Logger::debug("[handle_not_working] Cleaned db");

        // Logout
        self.torrent_manager.logout().await.context("[handle_not_working] Failed to logout of torrent client")?;

        Ok(())
    }

    /**
     * Strike torrents
     */
    fn strike_torrents(&self, strike_utils: &mut StrikeUtils, torrents_criteria: &HashMap<String, (Torrent, bool)>) -> Result<Vec<Torrent>, anyhow::Error> {
        // Get torrent hashes of torrents that meet criteria
        let criteria_met_hashes: Vec<String> = torrents_criteria.values().filter(|(_, met)| *met).map(|(torrent, _)| torrent.hash().to_string()).collect();

        // Strike torrents that meet criteria
        strike_utils.strike(StrikeType::HandleNotWorking, criteria_met_hashes.clone()).context("[handle_not_working] Failed to strike hashes")?;

        // Get all strike stuff from the db for this job
        let strike_records = strike_utils.get_strikes(StrikeType::HandleNotWorking, Some(criteria_met_hashes)).context("[handle_not_working] Failed get strikes")?;

        // Get torrents that reached the strike limits
        let mut limit_reached_torrents: Vec<Torrent> = Vec::new();
        for strike_record in strike_records {
            if strike_record.is_limit_reached(self.config.jobs().handle_not_working().required_strikes(), self.config.jobs().handle_not_working().min_strike_days()) {
                if let Some(torrent_criteria) = torrents_criteria.get(strike_record.hash()) {
                    limit_reached_torrents.push(torrent_criteria.clone().0);
                } else {
                    Logger::warn(format!("[handle_not_working] Didn't find torrent criteria for torrent that reached strike limit: {}", strike_record.hash()).as_str());
                }
            }
        }
        Ok(limit_reached_torrents)
    }

    /**
     * Take action
     */
    async fn take_action(&self, torrents_criteria: &HashMap<String, (Torrent, bool)>, torrent: &Torrent) -> Result<(), anyhow::Error> {
        let mut is_any_not_meeting_criteria = false;
        for (torrent, is_criteria_met) in torrents_criteria.values() {
            if !*is_criteria_met && torrent.content_path() == torrent.content_path() {
                is_any_not_meeting_criteria = true;
                break;
            }
        }
        match ActionType::from_str(self.config.jobs().handle_not_working().action()) {
            ActionType::Test => {
                Logger::info("[handle_not_working] Action: Test");
                if is_any_not_meeting_criteria {
                    Logger::debug("[handle_not_working] At least 1 other torrent depends this torrents files");
                }
            }
            ActionType::Stop => {
                Logger::info("[handle_not_working] Action: Stopping torrent");
                if is_any_not_meeting_criteria {
                    Logger::debug("[handle_not_working] At least 1 other torrent depends this torrents files");
                }
                self.torrent_manager.stop_torrent(torrent.hash()).await.context("[handle_not_working] Failed to stop torrent")?;
            }
            ActionType::Delete => {
                if is_any_not_meeting_criteria {
                    Logger::info("[handle_not_working] Action: Deleting torrent but keeping files (at least 1 other torrent depends on them)");
                    self.torrent_manager.delete_torrent(torrent.hash(), false).await.context("[handle_not_working] Failed to delete torrent")?;
                } else {
                    Logger::info("[handle_not_working] Action: Deleting torrent and files");
                    self.torrent_manager.delete_torrent(torrent.hash(), true).await.context("[handle_not_working] Failed to delete torrent")?;
                }
            }
        }
        Ok(())
    }

    /**
     * Clean db
     */
    fn clean_db(&self, strike_utils: &mut StrikeUtils, torrents_criteria: &HashMap<String, (Torrent, bool)>) -> Result<(), anyhow::Error> {
        let mut hashes_to_remove: Vec<String> = Vec::new();

        let strike_records = strike_utils
            .get_strikes(StrikeType::HandleNotWorking, None)
            .context("[handle_not_working] Failed to get all strikes for HandleNotWorking")?;
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

        Logger::trace(format!("[handle_not_working] Deleting {} hashes", hashes_to_remove.len()).as_str());

        strike_utils.delete(StrikeType::HandleNotWorking, hashes_to_remove).context("[handle_not_working] Failed to delete hashes")?;

        Ok(())
    }

    /**
     * Send notification
     */
    async fn send_notification(&self, discord_webhook_utils: &DiscordWebhookUtils, torrent: &Torrent, trackers: &Vec<Tracker>) -> Result<(), anyhow::Error> {
        if !discord_webhook_utils.is_notifications_enabled() {
            return Ok(());
        }

        let total_size_gib = format!("{:.2}", (*torrent.total_size() / 1024 / 1024) as f32 / 1024.0);
        let total_size_gb = format!("{:.2}", (*torrent.total_size() / 1000 / 1000) as f32 / 1000.0);

        let seeding_days = format!("{:.2}", (*torrent.seeding_time() / 60 / 60) as f32 / 24.0);

        let added_on_str = match Local.timestamp_opt(*torrent.added_on(), 0).single() {
            Some(datetime_local) => datetime_local.format("%Y-%m-%d %H:%M:%S").to_string(),
            None => String::from("Failed getting datetime"),
        };
        let completed_on_str = match *torrent.completion_on() {
            -1 => String::from("Not completed"),
            _ => match Local.timestamp_opt(*torrent.completion_on(), 0).single() {
                Some(datetime_local) => datetime_local.format("%Y-%m-%d %H:%M:%S").to_string(),
                None => String::from("Failed getting datetime"),
            },
        };

        let mut fields: Vec<EmbedField> = Vec::new();
        for tracker in trackers {
            let tracker_status_str = match TrackerStatus::from_int(*tracker.status()) {
                Ok(tracker_status) => tracker_status.to_string(),
                Err(e) => {
                    Logger::warn(e.as_str());
                    tracker.status().to_string()
                }
            };
            fields.push(EmbedField {
                name: String::from("Tracker"),
                value: format!("URL: {}\nStatus: {}\nMessage: {}", tracker.url(), tracker_status_str, tracker.msg()),
                inline: false,
            });
        }
        fields.extend(vec![
            EmbedField {
                name: String::from("Action"),
                value: self.config.jobs().handle_not_working().action().to_string(),
                inline: true,
            },
            EmbedField {
                name: String::from("Category"),
                value: torrent.category().to_string(),
                inline: true,
            },
            EmbedField {
                name: String::from("Tags"),
                value: torrent.tags().to_string(),
                inline: true,
            },
            EmbedField {
                name: String::from("Total Size"),
                value: format!("{total_size_gib}GiB | {total_size_gb}GB"),
                inline: true,
            },
            EmbedField {
                name: String::from("Ratio"),
                value: format!("{:.2}", torrent.ratio()),
                inline: true,
            },
            EmbedField {
                name: String::from("Seeding days"),
                value: seeding_days.to_string(),
                inline: true,
            },
            EmbedField {
                name: String::from("Added"),
                value: added_on_str,
                inline: true,
            },
            EmbedField {
                name: String::from("Completed"),
                value: completed_on_str,
                inline: true,
            },
        ]);

        discord_webhook_utils.send_webhook_embed(torrent.name(), "Found not working torrent", fields).await
    }

    /**
     * Is criteria met
     */
    async fn is_criteria_met(&self, torrent: &Torrent, trackers: &Vec<Tracker>) -> bool {
        // Uncompleted
        if *torrent.completion_on() == -1 {
            Logger::trace(format!("[handle_not_working] Torrent doesn't meet criteria (uncompleted): ({}) {}", torrent.hash(), torrent.name(),).as_str());
            return false;
        }
        // Protection tag
        if torrent.tags().contains(self.config.jobs().handle_not_working().protection_tag()) {
            Logger::trace(format!("[handle_not_working] Torrent doesn't meet criteria (protection tag): ({}) {}", torrent.hash(), torrent.name(),).as_str());
            return false;
        }
        // Stopped torrent
        if vec![
            TorrentState::PausedUP.as_string(),
            TorrentState::PausedDL.as_string(),
            TorrentState::StoppedUP.as_string(),
            TorrentState::StoppedDL.as_string(),
        ]
        .contains(&torrent.state().to_string())
        {
            Logger::trace(format!("[handle_not_working] Torrent doesn't meet criteria (stopped): ({}) {}", torrent.hash(), torrent.name(),).as_str());
            return false;
        }
        // Working trackers
        for tracker in trackers {
            match TrackerStatus::from_int(*tracker.status()) {
                Ok(tracker_status) => {
                    if matches!(tracker_status, TrackerStatus::Working) {
                        Logger::trace(format!("[handle_not_working] Torrent doesn't meet criteria (at least 1 working tracker): ({}) {}", torrent.hash(), torrent.name(),).as_str());
                        return false;
                    }
                }
                Err(e) => {
                    Logger::error(
                        format!(
                            "[handle_not_working] Torrent doesn't meet criteria (error while getting torrent tracker status): ({}) {}: {}",
                            torrent.hash(),
                            torrent.name(),
                            e,
                        )
                        .as_str(),
                    );
                    return false;
                }
            }
        }
        // All good
        Logger::trace(format!("[handle_not_working] Torrent meets criteria: ({}) {}", torrent.hash(), torrent.name()).as_str());
        true
    }
}
