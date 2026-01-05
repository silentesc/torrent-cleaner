use tokio::signal::unix::{SignalKind, signal};

use crate::{logger::enums::category::Category, setup::Setup};

mod config;
mod job_manager;
mod jobs;
mod logger;
mod setup;
mod torrent_clients;
mod utils;

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
    let job_manager = match Setup::setup().await {
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
