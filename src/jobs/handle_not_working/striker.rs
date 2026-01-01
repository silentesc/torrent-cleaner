use std::collections::HashMap;

use anyhow::Context;

use crate::{
    config::Config,
    jobs::{enums::strike_type::StrikeType, utils::strike_utils::StrikeUtils},
    logger::enums::category::Category,
    torrent_clients::models::torrent::Torrent,
    warn,
};

pub struct Striker;

impl Striker {
    /**
     * Strike torrents
     */
    pub fn strike_torrents(strike_utils: &mut StrikeUtils, torrents_criteria: &HashMap<String, (Torrent, bool)>, config: &Config) -> Result<Vec<Torrent>, anyhow::Error> {
        // Get torrent hashes of torrents that meet criteria
        let criteria_met_hashes: Vec<String> = torrents_criteria.values().filter(|(_, met)| *met).map(|(torrent, _)| torrent.hash().to_string()).collect();

        // Strike torrents that meet criteria
        strike_utils.strike(&StrikeType::HandleNotWorking, criteria_met_hashes.clone()).context("Failed to strike hashes")?;

        // Get all strike stuff from the db for this job
        let strike_records = strike_utils.get_strikes(&StrikeType::HandleNotWorking, Some(criteria_met_hashes)).context("Failed get strikes")?;

        // Get torrents that reached the strike limits
        let mut limit_reached_torrents: Vec<Torrent> = Vec::new();
        for strike_record in strike_records {
            if strike_record.is_limit_reached(config.jobs().handle_not_working().required_strikes(), config.jobs().handle_not_working().min_strike_days()) {
                if let Some(torrent_criteria) = torrents_criteria.get(strike_record.hash()) {
                    limit_reached_torrents.push(torrent_criteria.clone().0);
                } else {
                    warn!(Category::HandleNotWorking, "Didn't find torrent criteria for torrent that reached strike limit: {}", strike_record.hash(),);
                }
            }
        }
        Ok(limit_reached_torrents)
    }
}
