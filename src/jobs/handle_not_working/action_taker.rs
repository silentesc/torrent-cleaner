use std::{collections::HashMap, sync::Arc};

use anyhow::Context;

use crate::{
    config::Config,
    jobs::enums::action_type::ActionType,
    logger::logger::Logger,
    torrent_clients::{models::torrent::Torrent, torrent_manager::TorrentManager},
};

pub struct ActionTaker;

impl ActionTaker {
    /**
     * Take action
     */
    pub async fn take_action(torrent_manager: Arc<TorrentManager>, torrents_criteria: &HashMap<String, (Torrent, bool)>, torrent: &Torrent, config: &Config) -> Result<(), anyhow::Error> {
        let mut is_any_not_meeting_criteria = false;
        for (torrent, is_criteria_met) in torrents_criteria.values() {
            if !*is_criteria_met && torrent.content_path() == torrent.content_path() {
                is_any_not_meeting_criteria = true;
                break;
            }
        }
        match ActionType::from_str(config.jobs().handle_not_working().action()) {
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
                torrent_manager.stop_torrent(torrent.hash()).await.context("[handle_not_working] Failed to stop torrent")?;
            }
            ActionType::Delete => {
                if is_any_not_meeting_criteria {
                    Logger::info("[handle_not_working] Action: Deleting torrent but keeping files (at least 1 other torrent depends on them)");
                    torrent_manager.delete_torrent(torrent.hash(), false).await.context("[handle_not_working] Failed to delete torrent")?;
                } else {
                    Logger::info("[handle_not_working] Action: Deleting torrent and files");
                    torrent_manager.delete_torrent(torrent.hash(), true).await.context("[handle_not_working] Failed to delete torrent")?;
                }
            }
        }
        Ok(())
    }
}
