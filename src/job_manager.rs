use std::{sync::Arc, time::Duration};

use tokio::{sync::Mutex, time::sleep};

use crate::{
    config::Config,
    jobs::{handle_forgotten::HandleForgotten, handle_not_working::HandleNotWorking, handle_orphaned::HandleOrphaned},
    logger::logger::Logger,
    torrent_clients::torrent_manager::TorrentManager,
};

pub struct JobManager {
    config: Config,
    torrent_manager: Arc<TorrentManager>,
    torrents_path: String,
    media_path: String,
    job_lock: Arc<Mutex<()>>,
}

impl JobManager {
    pub fn new(config: Config, torrent_manager: Arc<TorrentManager>, torrents_path: String, media_path: String) -> Self {
        Self {
            config,
            torrent_manager,
            media_path,
            torrents_path,
            job_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn setup(&self) {
        let handle_forgotten = HandleForgotten::new(self.torrent_manager.clone(), self.media_path.clone(), self.config.clone());
        let handle_not_working = HandleNotWorking::new(self.torrent_manager.clone(), self.config.clone());
        let handle_orphaned = HandleOrphaned::new(self.torrent_manager.clone(), self.config.clone(), self.torrents_path.clone());

        // Handle Forgotten Job
        if self.config.jobs().handle_forgotten().interval_hours() != -1 {
            let lock_for_handle_forgotten = self.job_lock.clone();
            let config_for_handle_forgotten = self.config.clone();
            tokio::spawn(async move {
                Logger::info(format!("[job_manager] Set up handle_forgotten, next run in {} hours", config_for_handle_forgotten.jobs().handle_forgotten().interval_hours()).as_str());
                sleep(Duration::from_hours(config_for_handle_forgotten.jobs().handle_forgotten().interval_hours() as u64)).await;
                loop {
                    {
                        // Wait for lock
                        let _guard = lock_for_handle_forgotten.lock().await;
                        // Execute job
                        Logger::info("[job_manager] Starting handle_forgotten...");
                        if let Err(e) = handle_forgotten.run().await {
                            Logger::error(format!("[job_manager] Failed to run handle_forgotten: {:#}", e).as_str());
                        }
                        Logger::info(format!("[job_manager] handle_forgotten finished, next run in {} hours", config_for_handle_forgotten.jobs().handle_forgotten().interval_hours()).as_str());
                    }
                    // Sleep
                    sleep(Duration::from_hours(config_for_handle_forgotten.jobs().handle_forgotten().interval_hours() as u64)).await;
                }
            });
        }
        // Handle Not Working Job
        if self.config.jobs().handle_not_working().interval_hours() != -1 {
            let lock_for_handle_not_working = self.job_lock.clone();
            let config_for_handle_not_working = self.config.clone();
            tokio::spawn(async move {
                Logger::info(
                    format!(
                        "[job_manager] Set up handle_not_working, next run in {} hours",
                        config_for_handle_not_working.jobs().handle_not_working().interval_hours()
                    )
                    .as_str(),
                );
                sleep(Duration::from_hours(config_for_handle_not_working.jobs().handle_not_working().interval_hours() as u64)).await;
                loop {
                    {
                        // Wait for lock
                        let _guard = lock_for_handle_not_working.lock().await;
                        // Execute job
                        Logger::info("[job_manager] Starting handle_not_working...");
                        if let Err(e) = handle_not_working.run().await {
                            Logger::error(format!("[job_manager] Failed to run handle_not_working: {:#}", e).as_str());
                        }
                        Logger::info(
                            format!(
                                "[job_manager] handle_not_working finished, next run in {} hours",
                                config_for_handle_not_working.jobs().handle_not_working().interval_hours()
                            )
                            .as_str(),
                        );
                    }
                    // Sleep
                    sleep(Duration::from_hours(config_for_handle_not_working.jobs().handle_not_working().interval_hours() as u64)).await;
                }
            });
        }
        // Handle Orphaned Job
        if self.config.jobs().handle_orphaned().interval_hours() != -1 {
            let lock_for_handle_orphaned = self.job_lock.clone();
            let config_for_handle_orphaned = self.config.clone();
            tokio::spawn(async move {
                Logger::info(format!("[job_manager] Set up handle_orphaned, next run in {} hours", config_for_handle_orphaned.jobs().handle_orphaned().interval_hours()).as_str());
                sleep(Duration::from_hours(config_for_handle_orphaned.jobs().handle_orphaned().interval_hours() as u64)).await;
                loop {
                    {
                        // Wait for lock
                        let _guard = lock_for_handle_orphaned.lock().await;
                        // Execute job
                        Logger::info("[job_manager] Starting handle_orphaned...");
                        if let Err(e) = handle_orphaned.run().await {
                            Logger::error(format!("[job_manager] Failed to run handle_orphaned: {:#}", e).as_str());
                        }
                        Logger::info(format!("[job_manager] handle_orphaned finished, next run in {} hours", config_for_handle_orphaned.jobs().handle_orphaned().interval_hours()).as_str());
                    }
                    // Sleep
                    sleep(Duration::from_hours(config_for_handle_orphaned.jobs().handle_orphaned().interval_hours() as u64)).await;
                }
            });
        }
    }

    /* Getter */

    pub fn job_lock(&self) -> Arc<Mutex<()>> {
        self.job_lock.clone()
    }
}
