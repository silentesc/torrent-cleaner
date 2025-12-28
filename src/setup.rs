use std::{env, fs, path::Path, sync::Arc};

use crate::{
    config::Config,
    job_manager::JobManager,
    jobs::utils::strike_utils::StrikeUtils,
    logger::{
        enums::{category::Category, log_level::LogLevel},
        logger::Logger,
    },
    torrent_clients::{adapters::qbittorrent::Qbittorrent, enums::any_client::AnyClient, torrent_manager::TorrentManager},
};

pub struct Setup;

impl Setup {
    pub fn setup() -> Result<JobManager, anyhow::Error> {
        // Setup logging
        Setup::setup_logging();
        Logger::debug(Category::Setup, "Logger has been loaded");

        const APP_NAME: &str = env!("CARGO_PKG_NAME");
        const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
        Logger::info(Category::Setup, format!("Running {} v{}", APP_NAME, APP_VERSION).as_str());

        // Load env variables
        let torrents_path = match env::var("TORRENTS_PATH") {
            Ok(torrents_path) => torrents_path,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to get TORRENTS_PATH env variable: {:#}", e));
            }
        };

        // Setup Config
        let config = Setup::get_config()?;
        Logger::debug(Category::Setup, "Config has been loaded");

        // Create strike utils table
        if let Err(e) = Setup::check_create_db() {
            return Err(anyhow::anyhow!("Failed to check create db: {:#}", e));
        }

        // Setup torrent_manager
        let torrent_manager = match Setup::setup_torrent_manager(config.clone()) {
            Ok(torrent_manager) => torrent_manager,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to setup torrent_manager: {:#}", e));
            }
        };

        // Setup jobs
        let job_manager = JobManager::new(config.clone(), torrent_manager.clone(), torrents_path);
        job_manager.setup();

        Ok(job_manager)
    }

    fn setup_logging() {
        let log_level = match env::var("LOG_LEVEL") {
            Ok(log_level) => log_level,
            Err(e) => {
                Logger::error(Category::Setup, format!("Failed to get log_level env variable, using default (info): {:#}", e).as_str());
                LogLevel::Info.to_string()
            }
        };
        Logger::set_log_level(LogLevel::from_string(log_level.as_str()));
    }

    fn get_config() -> Result<Config, anyhow::Error> {
        let config_path = "/config/config.json";
        if !Path::new(config_path).exists() {
            let default_config = Config::default();
            match serde_json::to_string_pretty(&default_config) {
                Ok(json) => {
                    if let Err(e) = fs::write(config_path, json) {
                        return Err(anyhow::anyhow!("Failed to write default config json string to config file: {:#}", e));
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to convert default config into pretty string: {:#}", e));
                }
            }
        }
        let config: Config;
        match fs::read_to_string(config_path) {
            Ok(contents) => {
                config = match serde_json::from_str(&contents) {
                    Ok(config) => config,
                    Err(e) => {
                        return Err(anyhow::anyhow!("Failed to create config object from config file contents: {:#}", e));
                    }
                };
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to read string from config file: {:#}", e));
            }
        }
        Ok(config)
    }

    fn check_create_db() -> Result<(), anyhow::Error> {
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

    fn setup_torrent_manager(config: Config) -> Result<Arc<TorrentManager>, anyhow::Error> {
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
