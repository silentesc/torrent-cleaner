use std::env;

use tokio::signal::unix::{SignalKind, signal};

use crate::{job_manager::JobManager, logger::logger::Logger, setup::Setup};

mod torrent_clients {
    pub mod torrent_manager;
    pub mod adapters {
        pub mod qbittorrent;
    }
    pub mod enums {
        pub mod any_client;
        pub mod torrent_state;
        pub mod tracker_status;
    }
    pub mod models {
        pub mod torrent;
        pub mod tracker;
    }
    pub mod traits {
        pub mod torrent_client;
    }
}

mod logger {
    pub mod logger;
    pub mod enums {
        pub mod log_level;
    }
}

mod jobs {
    pub mod enums {
        pub mod action_type;
        pub mod strike_type;
    }
    pub mod utils {
        pub mod discord_webhook_utils;
        pub mod file_utils;
        pub mod strike_utils;
    }
    pub mod handle_forgotten;
    pub mod handle_not_working;
    pub mod handle_orphaned;
}

mod config;
mod job_manager;
mod setup;

#[tokio::main]
async fn main() {
    let mut sigint = match signal(SignalKind::interrupt()) {
        Ok(sigint) => sigint,
        Err(e) => {
            Logger::error(format!("Failed to setup sigint signal: {:#}", e).as_str());
            return;
        }
    };
    let mut sigterm = match signal(SignalKind::terminate()) {
        Ok(sigterm) => sigterm,
        Err(e) => {
            Logger::error(format!("Failed to setup sigterm signal: {:#}", e).as_str());
            return;
        }
    };

    // Setup logging
    Setup::setup_logging();

    // Load env variables
    let torrents_path = match env::var("TORRENTS_PATH") {
        Ok(torrents_path) => torrents_path,
        Err(e) => {
            Logger::error(format!("Failed to get TORRENTS_PATH env variable: {:#}", e).as_str());
            return;
        }
    };
    let media_path = match env::var("MEDIA_PATH") {
        Ok(media_path) => media_path,
        Err(e) => {
            Logger::error(format!("Failed to get MEDIA_PATH env variable: {:#}", e).as_str());
            return;
        }
    };

    // Setup Config
    let config = match Setup::get_config() {
        Ok(config) => config,
        Err(error_message) => {
            Logger::error(error_message.as_str());
            return;
        }
    };
    Logger::debug("Config has been loaded");

    // Create strike utils table
    if let Err(e) = Setup::check_create_db() {
        Logger::error(format!("Failed to check create db: {:#}", e).as_str());
        return;
    }

    // Setup torrent_manager
    let torrent_manager = match Setup::setup_torrent_manager(config.clone()) {
        Ok(torrent_manager) => torrent_manager,
        Err(e) => {
            Logger::error(format!("Failed to setup torrent_manager: {:#}", e).as_str());
            return;
        }
    };

    // Login to torrent client
    if let Err(e) = torrent_manager.login().await {
        Logger::error(format!("Failed to login: {:#}", e).as_str());
        return;
    }

    // Setup jobs
    let job_manager = JobManager::new(config.clone(), torrent_manager.clone(), torrents_path.clone(), media_path.clone());
    job_manager.setup();

    // Wait for signal
    tokio::select! {
        _ = sigint.recv() => {
            Logger::info("Received sigint");
        }
        _ = sigterm.recv() => {
            Logger::info("Received sigterm");
        }
    };

    // Cleanup after shutdown
    match torrent_manager.logout().await {
        Ok(_) => Logger::info("Logged out of qbittorrent"),
        Err(e) => Logger::error(format!("Failed to logout: {:#}", e).as_str()),
    }

    Logger::info("Checking if any jobs are running and waiting if there are any...");

    // Try to wait until all jobs are finished
    let _ = job_manager.job_lock().lock().await;

    Logger::info("All jobs are done, graceful shutdown was successful");
}
