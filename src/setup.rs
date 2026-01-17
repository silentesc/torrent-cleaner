use std::{env, fs, path::Path, sync::Arc};

use crate::{
    config::Config,
    debug, error, info,
    job_manager::JobManager,
    logger::{
        enums::{category::Category, log_level::LogLevel},
        logger::Logger,
    },
    torrent_clients::{adapters::qbittorrent::Qbittorrent, enums::any_client::AnyClient, torrent_manager::TorrentManager},
    utils::db_manager::DbManager,
};

pub struct Setup;

impl Setup {
    pub async fn setup() -> Result<JobManager, anyhow::Error> {
        // Setup logging
        Setup::setup_logging();
        debug!(Category::Setup, "Logger has been loaded");

        const APP_NAME: &str = env!("CARGO_PKG_NAME");
        const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
        info!(Category::Setup, "Running {} v{}", APP_NAME, APP_VERSION);

        // Load env variables
        let torrents_path = match env::var("TORRENTS_PATH") {
            Ok(torrents_path) => torrents_path,
            Err(e) => {
                anyhow::bail!("Failed to get TORRENTS_PATH env variable: {:#}", e);
            }
        };

        // Setup Config
        let config = Setup::get_config()?;
        debug!(Category::Setup, "Config has been loaded");

        // Create strike utils table
        if let Err(e) = DbManager::check_create_tables() {
            anyhow::bail!("Failed to check create db: {:#}", e);
        }

        // Setup torrent_manager
        let torrent_manager = match Setup::setup_torrent_manager(config.clone()) {
            Ok(torrent_manager) => torrent_manager,
            Err(e) => {
                anyhow::bail!("Failed to setup torrent_manager: {:#}", e);
            }
        };

        // Test torrent_manager
        info!(Category::Setup, "Testing torrent client (login/logout)");
        torrent_manager.login().await?;
        torrent_manager.logout().await?;

        // Setup jobs
        let job_manager = JobManager::new(config.clone(), torrent_manager.clone(), torrents_path);
        job_manager.setup();

        Ok(job_manager)
    }

    fn setup_logging() {
        let log_level = match env::var("LOG_LEVEL") {
            Ok(log_level) => log_level,
            Err(e) => {
                error!(Category::Setup, "Failed to get log_level env variable, using default (info): {:#}", e);
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
                        anyhow::bail!("Failed to write default config json string to config file: {:#}", e);
                    }
                }
                Err(e) => {
                    anyhow::bail!("Failed to convert default config into pretty string: {:#}", e);
                }
            }
        }
        let config: Config = match fs::read_to_string(config_path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    anyhow::bail!("Failed to create config object from config file contents: {:#}", e);
                }
            },
            Err(e) => {
                anyhow::bail!("Failed to read string from config file: {:#}", e);
            }
        };
        Ok(config)
    }

    fn setup_torrent_manager(config: Config) -> Result<Arc<TorrentManager>, anyhow::Error> {
        let torrent_manager = match config.torrent_client().client().to_lowercase().as_str() {
            "qbittorrent" => {
                let qbittorrent_client = match Qbittorrent::new(config.torrent_client().base_url(), config.torrent_client().username(), config.torrent_client().password()) {
                    Ok(q) => q,
                    Err(e) => {
                        anyhow::bail!("Failed to create qbittorrent: {:#}", e);
                    }
                };
                Arc::new(TorrentManager::new(AnyClient::Qbittorrent(qbittorrent_client)))
            }
            _ => {
                anyhow::bail!("No client specified");
            }
        };
        Ok(torrent_manager)
    }
}
