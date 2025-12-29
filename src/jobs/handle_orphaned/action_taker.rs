use std::{fs, path::Path};

use crate::{
    config::Config,
    jobs::enums::action_type::ActionType,
    logger::{enums::category::Category, logger::Logger},
};

pub struct ActionTaker;

impl ActionTaker {
    /**
     * Take action
     */
    pub fn take_action(path: &Path, config: &Config) -> Result<(), anyhow::Error> {
        let action_type = ActionType::from_str(config.jobs().handle_orphaned().action())?;
        match action_type {
            ActionType::Test => {
                Logger::info(Category::HandleOrphaned, "Action: Test");
            }
            ActionType::Stop => {
                Logger::warn(Category::HandleOrphaned, "Stop action not supported on orphaned files since files cannot be stopped");
            }
            ActionType::Delete => {
                if path.is_file() {
                    if let Err(e) = fs::remove_file(path) {
                        anyhow::bail!("Error deleting orphaned file ({}): {:#}", path.display(), e);
                    }
                } else if path.is_dir() {
                    if let Err(e) = fs::remove_dir(path) {
                        anyhow::bail!("Error deleting orphaned dir ({}): {:#}", path.display(), e);
                    }
                } else {
                    anyhow::bail!("Path is neither file or dir: {}", path.display());
                }
            }
        }

        Ok(())
    }
}
