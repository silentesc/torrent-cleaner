use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::Context;

use crate::{
    config::Config,
    jobs::utils::file_utils::FileUtils,
    logger::logger::Logger,
    torrent_clients::{models::torrent::Torrent, torrent_manager::TorrentManager},
};

pub struct Receiver;

impl Receiver {
    pub async fn get_torrents_criteria(torrent_manager: Arc<TorrentManager>, config: &Config, media_folder_path: &String) -> Result<HashMap<String, (Torrent, bool)>, anyhow::Error> {
        // Get torrents from torrent client
        Logger::debug("[handle_forgotten] Getting torrents...");
        let torrents = torrent_manager.get_all_torrents().await.context("[handle_forgotten] Failed to get all torrents")?;
        Logger::debug(format!("[handle_forgotten] Received {} torrents", torrents.len()).as_str());

        // Get inodes present in the media folder
        Logger::debug("[handle_forgotten] Getting inodes of media files...");
        let media_file_inodes = FileUtils::get_media_file_inodes(media_folder_path)?;
        Logger::debug(format!("[handle_forgotten] Received inodes of {} files", media_file_inodes.len()).as_str());

        // Check torrents for criteria
        Logger::debug("[handle_forgotten] Checking torrents for criteria...");
        let mut torrents_criteria: HashMap<String, (Torrent, bool)> = HashMap::new();
        for torrent in torrents.clone() {
            torrents_criteria.insert(torrent.hash().to_string(), (torrent.clone(), Receiver::is_criteria_met(&torrent, &media_file_inodes, config)));
        }
        Logger::debug("[handle_forgotten] Done checking torrents for criteria");

        Ok(torrents_criteria)
    }

    /**
     * Is criteria met
     */
    fn is_criteria_met(torrent: &Torrent, media_file_inodes: &HashSet<u64>, config: &Config) -> bool {
        // Uncompleted
        if *torrent.completion_on() == -1 {
            Logger::trace(format!("[handle_forgotten] Torrent doesn't meet criteria (uncompleted): ({}) {}", torrent.hash(), torrent.name(),).as_str());
            return false;
        }
        // Protection tag
        if torrent.tags().contains(config.jobs().handle_forgotten().protection_tag()) {
            Logger::trace(format!("[handle_forgotten] Torrent doesn't meet criteria (protection tag): ({}) {}", torrent.hash(), torrent.name(),).as_str());
            return false;
        }
        // Seed time
        let seeding_days = torrent.seeding_time() / 60 / 60 / 24;
        if seeding_days < config.jobs().handle_forgotten().min_seeding_days() as i64 {
            Logger::trace(
                format!(
                    "[handle_forgotten] Torrent doesn't meet criteria (minimum seed day limit {}/{}): ({}) {}",
                    seeding_days,
                    config.jobs().handle_forgotten().min_seeding_days(),
                    torrent.hash(),
                    torrent.name(),
                )
                .as_str(),
            );
            return false;
        }
        // Media library
        match FileUtils::is_torrent_in_media_library(&torrent.content_path(), media_file_inodes) {
            Ok(is_torrent_in_media_library) => {
                if is_torrent_in_media_library {
                    Logger::trace(format!("[handle_forgotten] Torrent doesn't meet criteria (has hardlink in media library): ({}) {}", torrent.hash(), torrent.name(),).as_str());
                    return false;
                }
            }
            Err(e) => {
                Logger::error(
                    format!(
                        "[handle_forgotten] Torrent doesn't meet criteria (error while checking if hardlink in media library): ({}) {}: {:#}",
                        torrent.hash(),
                        torrent.name(),
                        e,
                    )
                    .as_str(),
                );
                return false;
            }
        }
        Logger::trace(format!("[handle_forgotten] Torrent meets criteria: ({}) {}", torrent.hash(), torrent.name()).as_str());
        true
    }
}
