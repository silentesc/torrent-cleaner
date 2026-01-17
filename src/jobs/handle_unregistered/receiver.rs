use std::{collections::HashMap, sync::Arc};

use anyhow::Context;

use crate::{
    config::Config,
    debug,
    logger::enums::category::Category,
    torrent_clients::{
        enums::torrent_state::TorrentState,
        models::{torrent::Torrent, tracker::Tracker},
        torrent_manager::TorrentManager,
    },
    trace, warn,
};

pub struct Receiver;

impl Receiver {
    /**
     * Get torrent trackers
     */
    pub async fn get_torrent_trackers(torrent_manager: Arc<TorrentManager>, torrents: &Vec<Torrent>, config: &Config) -> Result<HashMap<String, Vec<Tracker>>, anyhow::Error> {
        // Get trackers
        let mut torrent_trackers: HashMap<String, Vec<Tracker>> = HashMap::new();
        for torrent in torrents {
            let trackers: Vec<Tracker> = torrent_manager
                .get_torrent_trackers(torrent.hash())
                .await
                .context(format!("Failed to get trackers for torrent: ({}) {}", torrent.hash(), torrent.name()))?
                .into_iter()
                .filter(|tracker| match tracker.url() {
                    "** [DHT] **" if *config.jobs().handle_unregistered().ignore_dht() => false,
                    "** [PeX] **" if *config.jobs().handle_unregistered().ignore_pex() => false,
                    "** [LSD] **" if *config.jobs().handle_unregistered().ignore_lsd() => false,
                    _ => true,
                })
                .collect();
            torrent_trackers.insert(torrent.hash().to_string(), trackers);
        }

        Ok(torrent_trackers)
    }

    /**
     * Get torrents and if they match criteria
     * Returns: HashMap<String, (Torrent, bool)> | HashMap<torrent_hash, (Torrent, is_criteria_met))>
     */
    pub async fn get_torrents_criteria(torrents: &Vec<Torrent>, torrent_trackers: &HashMap<String, Vec<Tracker>>, config: &Config) -> Result<HashMap<String, (Torrent, bool)>, anyhow::Error> {
        // Check torrents for criteria
        let mut torrents_criteria: HashMap<String, (Torrent, bool)> = HashMap::new();
        for torrent in torrents {
            if let Some(trackers) = torrent_trackers.get(torrent.hash()) {
                let is_criteria_met = Receiver::is_criteria_met(torrent, trackers, config).await.context("Failed to get criteria")?;
                torrents_criteria.insert(torrent.hash().to_string(), (torrent.clone(), is_criteria_met));
            } else {
                warn!(Category::HandleUnregistered, "Cannot get tracker for torrent: ({}) {}", torrent.hash(), torrent.name());
            }
        }

        Ok(torrents_criteria)
    }

    /**
     * Is criteria met
     */
    async fn is_criteria_met(torrent: &Torrent, trackers: &Vec<Tracker>, config: &Config) -> Result<bool, anyhow::Error> {
        // Uncompleted
        if *torrent.completion_on() == -1 {
            trace!(Category::HandleUnregistered, "Torrent doesn't meet criteria (uncompleted): ({}) {}", torrent.hash(), torrent.name(),);
            return Ok(false);
        }
        // Protection tag
        if torrent.tags().contains(config.jobs().handle_unregistered().protection_tag()) {
            trace!(Category::HandleUnregistered, "Torrent doesn't meet criteria (protection tag): ({}) {}", torrent.hash(), torrent.name(),);
            return Ok(false);
        }
        // Stopped torrent
        if [
            TorrentState::PausedUP.to_string(),
            TorrentState::PausedDL.to_string(),
            TorrentState::StoppedUP.to_string(),
            TorrentState::StoppedDL.to_string(),
        ]
        .contains(&torrent.state().to_string())
        {
            trace!(Category::HandleUnregistered, "Torrent doesn't meet criteria (stopped): ({}) {}", torrent.hash(), torrent.name(),);
            return Ok(false);
        }
        // Working trackers
        for tracker in trackers {
            if !tracker.is_unregistered() {
                trace!(
                    Category::HandleUnregistered,
                    "Torrent doesn't meet criteria (at least 1 tracker is not unregistered): ({}) {}",
                    torrent.hash(),
                    torrent.name(),
                );
                return Ok(false);
            }
        }
        // All good
        debug!(Category::HandleUnregistered, "Torrent meets criteria: ({}) {}", torrent.hash(), torrent.name());
        Ok(true)
    }
}
