use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use chrono::{DateTime, Local};
use reqwest::Url;
use walkdir::WalkDir;

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
    torrent_clients::torrent_manager::TorrentManager,
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
            true => Some(Url::parse(self.config.notification().discord_webhook_url()).context("[handle_orphaned] Failed to parse discord_webhook_url")?),
            false => None,
        };
        let discord_webhook_utils = DiscordWebhookUtils::new(discord_webhook_url);

        let mut strike_utils = StrikeUtils::new()?;

        // Get torrents from torrent client
        Logger::debug("[handle_orphaned] Getting torrents...");
        let torrents = self.torrent_manager.get_all_torrents().await.context("[handle_orphaned] Failed to get all torrents")?;
        Logger::debug(format!("[handle_orphaned] Received {} torrents", torrents.len()).as_str());

        // Get torrent paths
        Logger::debug("[handle_orphaned] Getting all paths in all torrents...");
        let mut torrent_paths: Vec<PathBuf> = Vec::new();
        for torrent in torrents {
            // Ignore incomplete
            if *torrent.completion_on() == -1 {
                continue;
            }
            for entry in WalkDir::new(torrent.content_path()) {
                let entry_result = entry.context("[handle_orphaned] Failed to get entry_result")?;
                // Check for file
                if entry_result.file_type().is_file() {
                    torrent_paths.push(entry_result.into_path());
                }
                // Check for empty dir
                else if entry_result.file_type().is_dir() {
                    let mut entries = fs::read_dir(entry_result.path()).context("[handle_orphaned] Failed to read dir")?;
                    if entries.next().is_none() {
                        torrent_paths.push(entry_result.into_path());
                    }
                }
            }
        }
        Logger::debug(format!("[handle_orphaned] Received {} torrent paths", torrent_paths.len()).as_str());

        // Get paths not present in any torrents
        Logger::debug("[handle_orphaned] Getting orphaned paths (files/folders that are not part of any torrent)...");
        let mut orphaned_path_strings: Vec<String> = Vec::new();
        for entry in WalkDir::new(self.torrents_path.as_str()) {
            let entry_result = entry.context("[handle_orphaned] Failed to get entry_result")?;
            // Check for file
            if entry_result.file_type().is_file() {
                let path_buf = entry_result.into_path();
                if torrent_paths.contains(&path_buf) {
                    continue;
                }
                let path_string = match path_buf.into_os_string().into_string() {
                    Ok(path_string) => path_string,
                    Err(os_string) => {
                        return Err(anyhow::anyhow!("[handle_orphaned] Failed to convert PathBuf into string: {}", os_string.display()));
                    }
                };
                orphaned_path_strings.push(path_string);
            }
            // Check for empty dir
            else if entry_result.file_type().is_dir() {
                let path_buf = entry_result.clone().into_path();
                if torrent_paths.contains(&path_buf) {
                    continue;
                }
                let mut entries = fs::read_dir(entry_result.path()).context("[handle_orphaned] Failed to read dir")?;
                if entries.next().is_some() {
                    continue;
                }
                let path_string = match path_buf.into_os_string().into_string() {
                    Ok(path_string) => path_string,
                    Err(os_string) => {
                        return Err(anyhow::anyhow!("[handle_orphaned] Failed to convert PathBuf into string: {}", os_string.display()));
                    }
                };
                orphaned_path_strings.push(path_string);
            }
        }
        Logger::debug(format!("[handle_orphaned] Received {} orphaned paths", orphaned_path_strings.len()).as_str());

        // Strike orphaned paths
        Logger::debug("[handle_orphaned] Striking orphaned paths...");
        let limit_reached_path_strings = self.strike_paths(&mut strike_utils, orphaned_path_strings)?;
        Logger::debug(format!("[handle_orphaned] Done striking, {} paths reached their limit. Action will be taken now", limit_reached_path_strings.len()).as_str());

        // Go through paths
        for path_string in limit_reached_path_strings.clone() {
            let path = Path::new(path_string.as_str());

            // Log
            Logger::info(format!("[handle_orphaned] Orphaned path: {}", path_string).as_str());

            // Notification
            self.send_notification(&discord_webhook_utils, path_string.as_str(), path)
                .await
                .context("[handle_orphaned] Failed to send notification")?;

            // Take action
            self.take_action(path);
        }

        // Remove paths that reached limit and were handled from db
        strike_utils.delete(StrikeType::HandleOrphaned, limit_reached_path_strings)?;

        Ok(())
    }

    /**
     * Strike paths
     */
    fn strike_paths(&self, strike_utils: &mut StrikeUtils, orphaned_path_strings: Vec<String>) -> Result<Vec<String>, anyhow::Error> {
        strike_utils.strike(StrikeType::HandleOrphaned, orphaned_path_strings).context("[handle_orphaned] Failed to strike orhaned paths")?;

        let strike_records = strike_utils.get_strikes(StrikeType::HandleOrphaned).context("[handle_orphaned] Failed get strikes")?;

        let mut limit_reached_path_strings: Vec<String> = Vec::new();
        for strike_record in strike_records {
            if strike_record.is_limit_reached(self.config.jobs().handle_orphaned().required_strikes(), self.config.jobs().handle_orphaned().min_strike_days()) {
                limit_reached_path_strings.push(strike_record.hash().to_string());
            }
        }

        Ok(limit_reached_path_strings)
    }

    /**
     * Take action
     */
    fn take_action(&self, path: &Path) {
        match ActionType::from_str(self.config.jobs().handle_orphaned().action()) {
            ActionType::Test => {
                Logger::info("[handle_orphaned] Action: Test");
            }
            ActionType::Stop => {
                Logger::warn("[handle_orphaned] Stop action not supported on orphaned files since files cannot be stopped");
            }
            ActionType::Delete => {
                if path.is_file() {
                    if let Err(e) = fs::remove_file(path) {
                        Logger::error(format!("Error deleting orphaned file ({}): {:#}", path.display(), e).as_str());
                    }
                } else if path.is_dir() {
                    if let Err(e) = fs::remove_dir(path) {
                        Logger::error(format!("Error deleting orphaned dir ({}): {:#}", path.display(), e).as_str());
                    }
                } else {
                    Logger::warn(format!("Path is neither file or dir: {}", path.display()).as_str());
                }
            }
        }
    }

    /**
     * Send notification
     */
    async fn send_notification(&self, discord_webhook_utils: &DiscordWebhookUtils, path_str: &str, path: &Path) -> Result<(), anyhow::Error> {
        if !discord_webhook_utils.is_notifications_enabled() {
            return Ok(());
        }

        let metadata = fs::metadata(path).context("[handle_orphaned] Failed to get file metadata")?;
        let file_size_gb_string = format!("{:.2}GB", (metadata.len() / 1000 / 1000) as f32 / 1000.0);
        let modified_time = metadata.modified().context("Failed to get file modified SystemTime")?;

        let modified_time: DateTime<Local> = modified_time.into();
        let modified_time: String = modified_time.format("%Y-%m-%d %H:%M:%S").to_string();

        let description = if path.is_file() {
            "Found orphaned **file**"
        } else if path.is_dir() {
            "Found orphaned **folder**"
        } else {
            Logger::warn(format!("Path is not file or folder: {}", path.display()).as_str());
            "Found orphaned path which isn't file or folder?"
        };

        let fields: Vec<EmbedField> = vec![
            EmbedField {
                name: String::from("Action"),
                value: self.config.jobs().handle_orphaned().action().to_string(),
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
