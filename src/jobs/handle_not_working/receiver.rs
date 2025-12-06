use std::{collections::HashMap, sync::Arc};

use anyhow::Context;

use crate::{
    config::Config,
    logger::logger::Logger,
    torrent_clients::{
        enums::{torrent_state::TorrentState, tracker_status::TrackerStatus},
        models::{torrent::Torrent, tracker::Tracker},
        torrent_manager::TorrentManager,
    },
};

pub struct Receiver;

impl Receiver {
    /**
     * Get torrent trackers
     */
    pub async fn get_torrent_trackers(torrent_manager: Arc<TorrentManager>, torrents: &Vec<Torrent>) -> Result<HashMap<String, Vec<Tracker>>, anyhow::Error> {
        // Get trackers
        let mut torrent_trackers: HashMap<String, Vec<Tracker>> = HashMap::new();
        for torrent in torrents {
            let trackers = torrent_manager
                .get_torrent_trackers(torrent.hash())
                .await
                .context(format!("[handle_not_working] Failed to get trackers for torrent: ({}) {}", torrent.hash(), torrent.name()))?;
            torrent_trackers.insert(torrent.hash().to_string(), trackers);
        }

        Ok(torrent_trackers)
    }

    /**
     * Get torrents criteria
     */
    pub async fn get_torrents_criteria(torrents: &Vec<Torrent>, torrent_trackers: &HashMap<String, Vec<Tracker>>, config: &Config) -> Result<HashMap<String, (Torrent, bool)>, anyhow::Error> {
        // Check torrents for criteria
        let mut torrents_criteria: HashMap<String, (Torrent, bool)> = HashMap::new();
        for torrent in torrents {
            if let Some(trackers) = torrent_trackers.get(torrent.hash()) {
                torrents_criteria.insert(torrent.hash().to_string(), (torrent.clone(), Receiver::is_criteria_met(&torrent, trackers, &config).await));
            } else {
                Logger::warn(format!("[handle_not_working] Cannot get tracker for torrent: ({}) {}", torrent.hash(), torrent.name()).as_str());
            }
        }

        Ok(torrents_criteria)
    }

    /**
     * Is criteria met
     */
    async fn is_criteria_met(torrent: &Torrent, trackers: &Vec<Tracker>, config: &Config) -> bool {
        // Uncompleted
        if *torrent.completion_on() == -1 {
            Logger::trace(format!("[handle_not_working] Torrent doesn't meet criteria (uncompleted): ({}) {}", torrent.hash(), torrent.name(),).as_str());
            return false;
        }
        // Protection tag
        if torrent.tags().contains(config.jobs().handle_not_working().protection_tag()) {
            Logger::trace(format!("[handle_not_working] Torrent doesn't meet criteria (protection tag): ({}) {}", torrent.hash(), torrent.name(),).as_str());
            return false;
        }
        // Stopped torrent
        if vec![
            TorrentState::PausedUP.to_string(),
            TorrentState::PausedDL.to_string(),
            TorrentState::StoppedUP.to_string(),
            TorrentState::StoppedDL.to_string(),
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
