use std::{collections::HashMap, sync::Arc};

use anyhow::Context;

use crate::{
    config::Config,
    jobs::utils::file_utils::FileUtils,
    logger::{enums::category::Category, logger::Logger},
    torrent_clients::{models::torrent::Torrent, torrent_manager::TorrentManager},
};

pub struct Receiver;

impl Receiver {
    pub async fn get_torrents_criteria(torrent_manager: Arc<TorrentManager>, config: &Config, torrents_path: &String) -> Result<HashMap<String, (Torrent, bool)>, anyhow::Error> {
        // Get torrents from torrent client
        Logger::debug(Category::HandleForgotten, "Getting torrents...");
        let torrents = torrent_manager.get_all_torrents().await.context("Failed to get all torrents")?;
        Logger::debug(Category::HandleForgotten, format!("Received {} torrents", torrents.len()).as_str());

        // Get known hardlinks
        Logger::debug(Category::HandleForgotten, "Getting known torrent hardlinks...");
        let known_hardlinks: HashMap<u64, u64> = FileUtils::get_known_hardlinks(torrents_path)?;
        Logger::debug(
            Category::HandleForgotten,
            format!("Found {} unique files ({} total) in torrent folder", known_hardlinks.len(), known_hardlinks.values().sum::<u64>()).as_str(),
        );

        // Check torrents for criteria
        Logger::debug(Category::HandleForgotten, "Checking torrents for criteria...");
        let mut torrents_criteria: HashMap<String, (Torrent, bool)> = HashMap::new();
        for torrent in &torrents {
            let is_criteria_met = Receiver::is_criteria_met(&torrent, &known_hardlinks, config)?;
            torrents_criteria.insert(torrent.hash().to_string(), (torrent.clone(), is_criteria_met));
        }
        Logger::debug(Category::HandleForgotten, "Done checking torrents for criteria");

        Ok(torrents_criteria)
    }

    /**
     * Is criteria met
     */
    fn is_criteria_met(torrent: &Torrent, known_hardlinks: &HashMap<u64, u64>, config: &Config) -> Result<bool, anyhow::Error> {
        // Uncompleted
        if *torrent.completion_on() == -1 {
            Logger::trace(Category::HandleForgotten, format!("Torrent doesn't meet criteria (uncompleted): ({}) {}", torrent.hash(), torrent.name()).as_str());
            return Ok(false);
        }
        // Protection tag
        if torrent.tags().contains(config.jobs().handle_forgotten().protection_tag()) {
            Logger::trace(Category::HandleForgotten, format!("Torrent doesn't meet criteria (protection tag): ({}) {}", torrent.hash(), torrent.name()).as_str());
            return Ok(false);
        }
        // Seed time
        let seeding_days = torrent.seeding_time() / 60 / 60 / 24;
        let min_seeding_days = config.jobs().handle_forgotten().min_seeding_days() as i64;
        if seeding_days < min_seeding_days {
            Logger::trace(
                Category::HandleForgotten,
                format!("Torrent doesn't meet criteria (minimum seed day limit {}/{}): ({}) {}", seeding_days, min_seeding_days, torrent.hash(), torrent.name()).as_str(),
            );
            return Ok(false);
        }
        // Media library
        let has_external_hardlinks = FileUtils::has_external_hardlinks(known_hardlinks, torrent.content_path())?;
        if has_external_hardlinks {
            Logger::trace(
                Category::HandleForgotten,
                format!("Torrent doesn't meet criteria (has external hardlink): ({}) {}", torrent.hash(), torrent.name()).as_str(),
            );
            return Ok(false);
        }

        Logger::trace(Category::HandleForgotten, format!("Torrent meets criteria: ({}) {}", torrent.hash(), torrent.name()).as_str());

        Ok(true)
    }
}
