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
        let handle_forgotten = Arc::new(HandleForgotten::new(self.torrent_manager.clone(), self.media_path.clone(), self.config.clone()));
        let handle_not_working = Arc::new(HandleNotWorking::new(self.torrent_manager.clone(), self.config.clone()));
        let handle_orphaned = Arc::new(HandleOrphaned::new(self.torrent_manager.clone(), self.config.clone(), self.torrents_path.clone()));

        self.spawn_job(
            String::from("handle_forgotten"),
            self.config.jobs().handle_forgotten().interval_hours(),
            Config::default().jobs().handle_forgotten().interval_hours(),
            handle_forgotten.clone(),
            |handler| async move {
                if let Err(e) = handler.run().await {
                    Logger::error(format!("[job_manager] Failed to run handle_forgotten: {:#}", e).as_str());
                }
            },
        );

        self.spawn_job(
            String::from("handle_not_working"),
            self.config.jobs().handle_not_working().interval_hours(),
            Config::default().jobs().handle_not_working().interval_hours(),
            handle_not_working.clone(),
            |handler| async move {
                if let Err(e) = handler.run().await {
                    Logger::error(format!("[job_manager] Failed to run handle_not_working: {:#}", e).as_str());
                }
            },
        );

        self.spawn_job(
            String::from("handle_orphaned"),
            self.config.jobs().handle_orphaned().interval_hours(),
            Config::default().jobs().handle_orphaned().interval_hours(),
            handle_orphaned.clone(),
            |handler| async move {
                if let Err(e) = handler.run().await {
                    Logger::error(format!("[job_manager] Failed to run handle_orphaned: {:#}", e).as_str());
                }
            },
        );
    }

    fn spawn_job<T, F, Fut>(&self, job_name: String, interval_hours: i32, default_interval_hours: i32, handler: Arc<T>, job_fn: F)
    where
        T: Send + Sync + 'static,
        F: (Fn(Arc<T>) -> Fut) + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        if interval_hours == -1 {
            return;
        }

        let lock = self.job_lock.clone();

        tokio::spawn(async move {
            Logger::info(format!("[job_manager] Set up {}, next run in {} hours", job_name, interval_hours).as_str());

            // Test/Sleep
            let mut interval_hours = interval_hours;
            if interval_hours != 0 {
                sleep(Duration::from_hours(interval_hours as u64)).await;
            } else {
                interval_hours = default_interval_hours;
            }

            loop {
                {
                    let _guard = lock.lock().await;
                    Logger::info(format!("[job_manager] Starting {}...", job_name).as_str());
                    job_fn(handler.clone()).await;
                    Logger::info(format!("[job_manager] {} finished, next run in {} hours", job_name, interval_hours).as_str());
                }
                sleep(Duration::from_hours(interval_hours as u64)).await;
            }
        });
    }

    /* Getter */

    pub fn job_lock(&self) -> Arc<Mutex<()>> {
        self.job_lock.clone()
    }
}
