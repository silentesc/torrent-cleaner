use anyhow::Context;

use crate::{
    config::Config,
    jobs::{enums::strike_type::StrikeType, utils::strike_utils::StrikeUtils},
};

pub struct Striker;

impl Striker {
    /**
     * Strike paths
     */
    pub fn strike_paths(strike_utils: &mut StrikeUtils, orphaned_path_strings: Vec<String>, config: &Config) -> Result<Vec<String>, anyhow::Error> {
        strike_utils
            .strike(StrikeType::HandleOrphaned, orphaned_path_strings.clone())
            .context("[handle_orphaned] Failed to strike orhaned paths")?;

        let strike_records = strike_utils.get_strikes(StrikeType::HandleOrphaned, Some(orphaned_path_strings)).context("[handle_orphaned] Failed get strikes")?;

        let mut limit_reached_path_strings: Vec<String> = Vec::new();
        for strike_record in strike_records {
            if strike_record.is_limit_reached(config.jobs().handle_orphaned().required_strikes(), config.jobs().handle_orphaned().min_strike_days()) {
                limit_reached_path_strings.push(strike_record.hash().to_string());
            }
        }

        Ok(limit_reached_path_strings)
    }
}
