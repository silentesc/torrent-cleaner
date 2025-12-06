use std::{env, fs, path::Path, sync::Arc};

use crate::{
    config::Config,
    jobs::utils::strike_utils::StrikeUtils,
    logger::{enums::log_level::LogLevel, logger::Logger},
    torrent_clients::{adapters::qbittorrent::Qbittorrent, enums::any_client::AnyClient, torrent_manager::TorrentManager},
};

pub struct Setup;

impl Setup {
    pub fn setup_logging() {
        let log_level = match env::var("LOG_LEVEL") {
            Ok(log_level) => log_level,
            Err(e) => {
                Logger::error(format!("Failed to get log_level env variable, using default (info): {:#}", e).as_str());
                LogLevel::Info.to_string()
            }
        };
        Logger::set_log_level(LogLevel::from_string(log_level.as_str()));
        Logger::debug("Logger has been loaded");
    }

    pub fn get_config() -> Result<Config, String> {
        let config_path = "/config/config.json";
        if !Path::new(config_path).exists() {
            let default_config = Config::default();
            match serde_json::to_string_pretty(&default_config) {
                Ok(json) => {
                    if let Err(e) = fs::write(config_path, json) {
                        return Err(format!("Failed to write default config json string to config file: {:#}", e));
                    }
                }
                Err(e) => {
                    return Err(format!("Failed to convert default config into pretty string: {:#}", e));
                }
            }
        }
        let config: Config;
        match fs::read_to_string(config_path) {
            Ok(contents) => {
                config = match serde_json::from_str(&contents) {
                    Ok(config) => config,
                    Err(e) => {
                        Logger::error(format!("Failed to create config object from config file contents, using default config instead: {:#}", e).as_str());
                        Config::default()
                    }
                };
            }
            Err(e) => {
                return Err(format!("Failed to read string from config file: {:#}", e));
            }
        }
        Ok(config)
    }

    pub fn check_create_db() -> Result<(), anyhow::Error> {
        // Create strike utils table
        match StrikeUtils::new() {
            Ok(mut strike_utils) => {
                if let Err(e) = strike_utils.check_create_tables() {
                    return Err(anyhow::anyhow!("Failed to create tables: {:#}", e));
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to init strike utils: {:#}", e));
            }
        }
        Ok(())
    }

    pub fn setup_torrent_manager(config: Config) -> Result<Arc<TorrentManager>, anyhow::Error> {
        let torrent_manager = match config.torrent_client().client().to_lowercase().as_str() {
            "qbittorrent" => {
                let qbittorrent_client = match Qbittorrent::new(config.torrent_client().base_url(), config.torrent_client().username(), config.torrent_client().password()) {
                    Ok(q) => q,
                    Err(e) => {
                        return Err(anyhow::anyhow!("Failed to create qbittorrent: {:#}", e));
                    }
                };
                Arc::new(TorrentManager::new(AnyClient::Qbittorrent(qbittorrent_client)))
            }
            _ => {
                return Err(anyhow::anyhow!("No client specified"));
            }
        };
        Ok(torrent_manager)
    }
}
