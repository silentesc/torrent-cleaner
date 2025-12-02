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
            file_utils::FileUtils,
            strike_utils::StrikeUtils,
        },
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
        Logger::info("[handle_forgotten] Job started");

        let discord_webhook_url = Url::parse(self.config.notification().discord_webhook_url()).context("[handle_forgotten]  Failed to parse discord_webhook_url")?;
        let discord_webhook_utils = DiscordWebhookUtils::new(discord_webhook_url);

        // Get torrents from torrent client
        Logger::debug("[handle_forgotten] Getting torrents...");
        let torrents = self.torrent_manager.get_all_torrents().await.context("[handle_forgotten] Failed to get all torrents")?;
        Logger::debug(format!("[handle_forgotten] Received {} torrents", torrents.len()).as_str());

        // Get inodes present in the media folder
        Logger::debug("[handle_forgotten] Getting inodes of media files...");
        let media_file_inodes = FileUtils::get_media_file_inodes(&self.media_folder_path);
        Logger::debug(format!("[handle_forgotten] Received inodes of {} files", media_file_inodes.len()).as_str());

        // Check torrents for criteria
        Logger::debug("[handle_forgotten] Checking torrents for criteria...");
        let mut torrents_criteria: HashMap<String, (Torrent, bool)> = HashMap::new();
        for torrent in torrents.clone() {
            torrents_criteria.insert(torrent.hash().to_string(), (torrent.clone(), self.is_criteria_met(torrent.clone(), &media_file_inodes)));
        }
        Logger::debug("[handle_forgotten] Done checking torrents for criteria");

        // Striking
        Logger::debug("[handle_forgotten] Striking torrents...");
        let mut strike_utils = StrikeUtils::new()?;
        let limit_reached_torrents = self.strike_torrents(&mut strike_utils, &torrents_criteria)?;
        Logger::debug(format!("[handle_forgotten] Done striking, {} torrents are forgotten", limit_reached_torrents.len()).as_str());

        // Go through torrents
        for torrent in limit_reached_torrents.clone() {
            // Log
            Logger::info(format!("[handle_forgotten] Torrent forgotten: {}", torrent.name()).as_str());

            // Notification
            self.send_notification(&discord_webhook_utils, torrent.clone())
                .await
                .context("[handle_forgotten] Failed to send notification")?;

            // Take action
            self.take_action(&torrents_criteria, &torrent.clone()).await?;
        }

        // Remove torrents that reached limit and were handled from db
        let limit_reached_torrent_hashes: Vec<String> = limit_reached_torrents.iter().map(|torrent| torrent.hash().to_string()).collect();
        strike_utils.delete(StrikeType::HandleForgotten, limit_reached_torrent_hashes)?;

        Logger::info("[handle_forgotten] Job finished");

        Ok(())
    }

    /**
     * Strike torrents
     */
    fn strike_torrents(&self, strike_utils: &mut StrikeUtils, torrents_criteria: &HashMap<String, (Torrent, bool)>) -> Result<Vec<Torrent>, anyhow::Error> {
        // Get torrent hashes of torrents that meet criteria
        let criteria_met_hashes: Vec<String> = torrents_criteria
            .clone()
            .values()
            .filter(|torrent_criteria| torrent_criteria.1)
            .map(|torrent_criteria| torrent_criteria.0.hash().to_string())
            .collect();

        // Strike torrents that meet criteria
        strike_utils
            .strike(StrikeType::HandleForgotten, criteria_met_hashes)
            .context("[handle_forgotten] Failed to strike hashes")?;

        // Get all strike stuff from the db for this job
        let strike_records = strike_utils.get_strikes(StrikeType::HandleForgotten).context("[handle_forgotten] Failed get strikes")?;

        // Get hashes that reached the strike limits
        let mut limit_reached_torrents: Vec<Torrent> = Vec::new();
        for strike_record in strike_records {
            if strike_record.is_limit_reached(self.config.jobs().handle_forgotten().required_strikes(), self.config.jobs().handle_forgotten().min_strike_days()) {
                if let Some(torrent_criteria) = torrents_criteria.get(strike_record.hash()) {
                    limit_reached_torrents.push(torrent_criteria.clone().0);
                } else {
                    Logger::warn(format!("Didn't find torrent criteria for torrent that reached strike limit: {}", strike_record.hash()).as_str());
                }
            }
        }
        Ok(limit_reached_torrents)
    }

    /**
     * Take action
     */
    async fn take_action(&self, torrents_criteria: &HashMap<String, (Torrent, bool)>, torrent: &Torrent) -> Result<(), anyhow::Error> {
        match ActionType::from_str(self.config.jobs().handle_forgotten().action()) {
            ActionType::Test => {
                Logger::info("[handle_forgotten] Action: Test");
            }
            ActionType::Stop => {
                Logger::info("[handle_forgotten] Action: Stopping torrent");
                self.torrent_manager.stop_torrent(torrent.hash()).await.context("[handle_forgotten] Failed to stop torrent")?;
            }
            ActionType::Delete => {
                let mut is_any_not_meeting_criteria = false;
                for torrent_criteria in torrents_criteria.values() {
                    if torrent_criteria.0.content_path() == torrent.content_path() && !torrent_criteria.1 {
                        is_any_not_meeting_criteria = true;
                        break;
                    }
                }
                if is_any_not_meeting_criteria {
                    Logger::info("[handle_forgotten] Action: Deleting torrent but keeping files (at least 1 other torrent depends on them)");
                    self.torrent_manager
                        .delete_torrent(torrent.hash(), false)
                        .await
                        .context("[handle_forgotten] Failed to delete torrent")?;
                } else {
                    Logger::info("[handle_forgotten] Action: Deleting torrent and files");
                    self.torrent_manager.delete_torrent(torrent.hash(), true).await.context("[handle_forgotten] Failed to delete torrent")?;
                }
            }
        }
        Ok(())
    }

    /**
     * Send notification
     */
    async fn send_notification(&self, discord_webhook_utils: &DiscordWebhookUtils, torrent_info: Torrent) -> Result<(), anyhow::Error> {
        let total_size_gib = format!("{:.2}", (*torrent_info.total_size() as f32) / 1024.0 / 1024.0 / 1024.0);
        let total_size_gb = format!("{:.2}", (*torrent_info.total_size() as f32) / 1000.0 / 1000.0 / 1000.0);

        let seeding_days = format!("{:.2}", (*torrent_info.seeding_time() as f32) / 60.0 / 60.0 / 24.0);

        let added_on_str = match Local.timestamp_opt(*torrent_info.added_on(), 0).single() {
            Some(datetime_local) => datetime_local.format("%Y-%m-%d %H:%M:%S").to_string(),
            None => String::from("Failed getting datetime"),
        };
        let completed_on_str = match *torrent_info.completion_on() {
            -1 => String::from("Not completed"),
            _ => match Local.timestamp_opt(*torrent_info.completion_on(), 0).single() {
                Some(datetime_local) => datetime_local.format("%Y-%m-%d %H:%M:%S").to_string(),
                None => String::from("Failed getting datetime"),
            },
        };

        let fields = vec![
            EmbedField {
                name: String::from("Tracker"),
                value: torrent_info.tracker().to_string(),
                inline: false,
            },
            EmbedField {
                name: String::from("Action"),
                value: self.config.jobs().handle_forgotten().action().to_string(),
                inline: true,
            },
            EmbedField {
                name: String::from("Category"),
                value: torrent_info.category().to_string(),
                inline: true,
            },
            EmbedField {
                name: String::from("Tags"),
                value: torrent_info.tags().to_string(),
                inline: true,
            },
            EmbedField {
                name: String::from("Total Size"),
                value: format!("{total_size_gib}GiB | {total_size_gb}GB"),
                inline: true,
            },
            EmbedField {
                name: String::from("Ratio"),
                value: format!("{:.2}", torrent_info.ratio()),
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
        ];
        discord_webhook_utils.send_webhook_embed(torrent_info.name(), "Found forgotten torrent", fields).await
    }

    /**
     * Is criteria met
     */
    fn is_criteria_met(&self, torrent_info: Torrent, media_file_inodes: &Vec<u64>) -> bool {
        // Uncompleted
        if *torrent_info.completion_on() == -1 {
            Logger::trace(format!("[handle_forgotten] Torrent doesn't meet criteria (uncompleted): ({}) {}", torrent_info.hash(), torrent_info.name(),).as_str());
            return false;
        }
        // Protection tag
        if torrent_info.tags().contains(self.config.jobs().handle_forgotten().protection_tag()) {
            Logger::trace(format!("[handle_forgotten] Torrent doesn't meet criteria (protection tag): ({}) {}", torrent_info.hash(), torrent_info.name(),).as_str());
            return false;
        }
        // Seed time
        let seeding_days = (torrent_info.seeding_time() / 60 / 60 / 24) as i32;
        if seeding_days < self.config.jobs().handle_forgotten().min_seeding_days() {
            Logger::trace(
                format!(
                    "[handle_forgotten] Torrent doesn't meet criteria (minimum seed day limit {}/{}): ({}) {}",
                    seeding_days,
                    self.config.jobs().handle_forgotten().min_seeding_days(),
                    torrent_info.hash(),
                    torrent_info.name(),
                )
                .as_str(),
            );
            return false;
        }
        // Media library
        match FileUtils::is_torrent_in_media_library(&torrent_info.content_path(), media_file_inodes) {
            Ok(is_torrent_in_media_library) => {
                if is_torrent_in_media_library {
                    Logger::trace(
                        format!(
                            "[handle_forgotten] Torrent doesn't meet criteria (has hardlink in media library): ({}) {}",
                            torrent_info.hash(),
                            torrent_info.name(),
                        )
                        .as_str(),
                    );
                    return false;
                }
            }
            Err(e) => {
                Logger::error(
                    format!(
                        "[handle_forgotten] Torrent doesn't meet criteria (error while checking if hardlink in media library): ({}) {}: {:#}",
                        torrent_info.hash(),
                        torrent_info.name(),
                        e,
                    )
                    .as_str(),
                );
                return false;
            }
        }
        Logger::trace(format!("[handle_forgotten] Torrent meets criteria: ({}) {}", torrent_info.hash(), torrent_info.name()).as_str());
        true
    }
}
