use std::{collections::HashMap, sync::Arc};

use anyhow::Context;

use crate::{
    config::Config,
    jobs::enums::action_type::ActionType,
    logger::{enums::category::Category, logger::Logger},
    torrent_clients::{models::torrent::Torrent, torrent_manager::TorrentManager},
};

pub struct ActionTaker;

impl ActionTaker {
    /**
     * Take action
     */
    pub async fn take_action(torrent_manager: Arc<TorrentManager>, torrents_criteria: &HashMap<String, (Torrent, bool)>, torrent: &Torrent, config: &Config) -> Result<(), anyhow::Error> {
        let mut is_any_not_meeting_criteria = false;
        for (t, is_criteria_met) in torrents_criteria.values() {
            if !*is_criteria_met && torrent.content_path() == t.content_path() {
                is_any_not_meeting_criteria = true;
                break;
            }
        }
        match ActionType::from_str(config.jobs().handle_not_working().action()) {
            ActionType::Test => {
                Logger::info(Category::HandleNotWorking, "Action: Test");
                if is_any_not_meeting_criteria {
                    Logger::debug(Category::HandleNotWorking, "At least 1 other torrent depends this torrents files");
                }
            }
            ActionType::Stop => {
                Logger::info(Category::HandleNotWorking, "Action: Stopping torrent");
                if is_any_not_meeting_criteria {
                    Logger::debug(Category::HandleNotWorking, "At least 1 other torrent depends this torrents files");
                }
                torrent_manager.stop_torrent(torrent.hash()).await.context("Failed to stop torrent")?;
            }
            ActionType::Delete => {
                if is_any_not_meeting_criteria {
                    Logger::info(Category::HandleNotWorking, "Action: Deleting torrent but keeping files (at least 1 other torrent depends on them)");
                    torrent_manager.delete_torrent(torrent.hash(), false).await.context("Failed to delete torrent")?;
                } else {
                    Logger::info(Category::HandleNotWorking, "Action: Deleting torrent and files");
                    torrent_manager.delete_torrent(torrent.hash(), true).await.context("Failed to delete torrent")?;
                }
            }
        }
        Ok(())
    }
}
