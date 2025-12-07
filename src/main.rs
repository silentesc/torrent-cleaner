use tokio::signal::unix::{SignalKind, signal};

use crate::{
    logger::{enums::category::Category, logger::Logger},
    setup::Setup,
};

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
        pub mod category;
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
    pub mod handle_forgotten {
        pub mod action_taker;
        pub mod handle_forgotten;
        pub mod notifier;
        pub mod receiver;
        pub mod striker;
    }
    pub mod handle_not_working {
        pub mod action_taker;
        pub mod handle_not_working;
        pub mod notifier;
        pub mod receiver;
        pub mod striker;
    }
    pub mod handle_orphaned {
        pub mod action_taker;
        pub mod handle_orphaned;
        pub mod notifier;
        pub mod receiver;
        pub mod striker;
    }
}

mod config;
mod job_manager;
mod setup;

#[tokio::main]
async fn main() {
    // Define signals
    let mut sigint = match signal(SignalKind::interrupt()) {
        Ok(sigint) => sigint,
        Err(e) => {
            Logger::error(Category::Setup, format!("Failed to setup sigint signal: {:#}", e).as_str());
            return;
        }
    };
    let mut sigterm = match signal(SignalKind::terminate()) {
        Ok(sigterm) => sigterm,
        Err(e) => {
            Logger::error(Category::Setup, format!("Failed to setup sigterm signal: {:#}", e).as_str());
            return;
        }
    };

    // Setup
    let job_manager = match Setup::setup() {
        Ok(job_manager) => job_manager,
        Err(e) => {
            Logger::error(Category::Setup, format!("{:#}", e).as_str());
            return;
        }
    };

    // Wait for signal
    tokio::select! {
        _ = sigint.recv() => {
            Logger::info(Category::Setup, "Received sigint");
        }
        _ = sigterm.recv() => {
            Logger::info(Category::Setup, "Received sigterm");
        }
    };

    // Cleanup after signal
    job_manager.wait_for_jobs_to_finish().await;

    Logger::info(Category::Setup, "Graceful shutdown successful");
}
