use tokio::signal::unix::{SignalKind, signal};

use crate::{logger::enums::category::Category, setup::Setup};

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
        pub mod torrent_file;
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
    pub mod handle_unlinked {
        pub mod action_taker;
        pub mod handle_unlinked;
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

mod utils {
    pub mod date_utils;
    pub mod db_manager;
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
            error!(Category::Setup, "Failed to setup sigint signal: {:#}", e);
            return;
        }
    };
    let mut sigterm = match signal(SignalKind::terminate()) {
        Ok(sigterm) => sigterm,
        Err(e) => {
            error!(Category::Setup, "Failed to setup sigterm signal: {:#}", e);
            return;
        }
    };

    // Setup
    let job_manager = match Setup::setup() {
        Ok(job_manager) => job_manager,
        Err(e) => {
            error!(Category::Setup, "{:#}", e);
            return;
        }
    };

    // Wait for signal
    tokio::select! {
        _ = sigint.recv() => {
            info!(Category::Setup, "Received sigint");
        }
        _ = sigterm.recv() => {
            info!(Category::Setup, "Received sigterm");
        }
    };

    // Cleanup after signal
    job_manager.wait_for_jobs_to_finish().await;

    info!(Category::Setup, "Graceful shutdown successful");
}
