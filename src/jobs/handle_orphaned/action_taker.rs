use std::{fs, path::Path};

use crate::{config::Config, jobs::enums::action_type::ActionType, logger::logger::Logger};

pub struct ActionTaker;

impl ActionTaker {
    /**
     * Take action
     */
    pub fn take_action(path: &Path, config: &Config) {
        match ActionType::from_str(config.jobs().handle_orphaned().action()) {
            ActionType::Test => {
                Logger::info("[handle_orphaned] Action: Test");
            }
            ActionType::Stop => {
                Logger::warn("[handle_orphaned] Stop action not supported on orphaned files since files cannot be stopped");
            }
            ActionType::Delete => {
                if path.is_file() {
                    if let Err(e) = fs::remove_file(path) {
                        Logger::error(format!("[handle_orphaned] Error deleting orphaned file ({}): {:#}", path.display(), e).as_str());
                    }
                } else if path.is_dir() {
                    if let Err(e) = fs::remove_dir(path) {
                        Logger::error(format!("[handle_orphaned] Error deleting orphaned dir ({}): {:#}", path.display(), e).as_str());
                    }
                } else {
                    Logger::warn(format!("[handle_orphaned] Path is neither file or dir: {}", path.display()).as_str());
                }
            }
        }
    }
}
