use std::{sync::Arc, time::Duration};

use reqwest::Url;
use tokio::{sync::Mutex, time::sleep};

use crate::{
    config::Config,
    jobs::{handle_not_working::handle_not_working::HandleNotWorking, handle_orphaned::handle_orphaned::HandleOrphaned, handle_unlinked::handle_unlinked::HandleUnlinked, utils::discord_webhook_utils::DiscordWebhookUtils},
    logger::{enums::category::Category, logger::Logger},
    torrent_clients::torrent_manager::TorrentManager,
};

pub struct JobManager {
    config: Config,
    torrent_manager: Arc<TorrentManager>,
    torrents_path: String,
    job_lock: Arc<Mutex<()>>,
}

impl JobManager {
    pub fn new(config: Config, torrent_manager: Arc<TorrentManager>, torrents_path: String) -> Self {
        Self {
            config,
            torrent_manager,
            torrents_path,
            job_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn setup(&self) {
        let handle_unlinked = Arc::new(HandleUnlinked::new(self.torrent_manager.clone(), self.config.clone(), self.torrents_path.clone()));
        let handle_not_working = Arc::new(HandleNotWorking::new(self.torrent_manager.clone(), self.config.clone()));
        let handle_orphaned = Arc::new(HandleOrphaned::new(self.torrent_manager.clone(), self.config.clone(), self.torrents_path.clone()));

        let discord_webhook_url = Some(self.config.notification().discord_webhook_url()).filter(|s| !s.is_empty()).and_then(|url_str| Url::parse(url_str).ok());

        self.spawn_job(
            String::from("handle_unlinked"),
            self.config.jobs().handle_unlinked().interval_hours(),
            Config::default().jobs().handle_unlinked().interval_hours(),
            *self.config.notification().on_job_error(),
            discord_webhook_url.clone(),
            handle_unlinked.clone(),
            |handler: Arc<HandleUnlinked>, notify_on_job_error: bool, discord_webhook_url: Option<Url>| async move {
                if let Err(e) = handler.run().await {
                    Logger::error(Category::JobManager, format!("Failed to run handle_unlinked: {:#}", e).as_str());
                    // Notify on discord
                    if notify_on_job_error {
                        let mut discord_webhook_utils = DiscordWebhookUtils::new(discord_webhook_url);
                        if let Err(e) = discord_webhook_utils.send_webhook_embed("Error", "`handle_unlinked` threw an error. Please check logs for more details.", vec![]).await {
                            Logger::error(Category::JobManager, format!("Error while sending discord webhook error message: {:#}", e).as_str());
                        }
                    }
                }
            },
        );

        self.spawn_job(
            String::from("handle_not_working"),
            self.config.jobs().handle_not_working().interval_hours(),
            Config::default().jobs().handle_not_working().interval_hours(),
            *self.config.notification().on_job_error(),
            discord_webhook_url.clone(),
            handle_not_working.clone(),
            |handler: Arc<HandleNotWorking>, notify_on_job_error: bool, discord_webhook_url: Option<Url>| async move {
                if let Err(e) = handler.run().await {
                    Logger::error(Category::JobManager, format!("Failed to run handle_not_working: {:#}", e).as_str());
                    // Notify on discord
                    if notify_on_job_error {
                        let mut discord_webhook_utils = DiscordWebhookUtils::new(discord_webhook_url);
                        if let Err(e) = discord_webhook_utils
                            .send_webhook_embed("Error", "`handle_not_working` threw an error. Please check logs for more details.", vec![])
                            .await
                        {
                            Logger::error(Category::JobManager, format!("Error while sending discord webhook error message: {:#}", e).as_str());
                        }
                    }
                }
            },
        );

        self.spawn_job(
            String::from("handle_orphaned"),
            self.config.jobs().handle_orphaned().interval_hours(),
            Config::default().jobs().handle_orphaned().interval_hours(),
            *self.config.notification().on_job_error(),
            discord_webhook_url.clone(),
            handle_orphaned.clone(),
            |handler: Arc<HandleOrphaned>, notify_on_job_error: bool, discord_webhook_url: Option<Url>| async move {
                if let Err(e) = handler.run().await {
                    Logger::error(Category::JobManager, format!("Failed to run handle_orphaned: {:#}", e).as_str());
                    // Notify on discord
                    if notify_on_job_error {
                        let mut discord_webhook_utils = DiscordWebhookUtils::new(discord_webhook_url);
                        if let Err(e) = discord_webhook_utils.send_webhook_embed("Error", "`handle_orphaned` threw an error. Please check logs for more details.", vec![]).await {
                            Logger::error(Category::JobManager, format!("Error while sending discord webhook error message: {:#}", e).as_str());
                        }
                    }
                }
            },
        );
    }

    fn spawn_job<T, F, Fut>(&self, job_name: String, interval_hours: i32, default_interval_hours: i32, notify_on_job_error: bool, discord_webhook_url: Option<Url>, handler: Arc<T>, job_fn: F)
    where
        T: Send + Sync + 'static,
        F: (Fn(Arc<T>, bool, Option<Url>) -> Fut) + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        if interval_hours == -1 {
            return;
        }

        let lock = self.job_lock.clone();

        tokio::spawn(async move {
            Logger::info(Category::JobManager, format!("Set up {}, next run in {} hours", job_name, interval_hours).as_str());

            // Test/Sleep
            let mut interval_hours = interval_hours;
            if interval_hours != 0 {
                sleep(Duration::from_hours(interval_hours as u64)).await;
            } else {
                interval_hours = default_interval_hours;
            }

            loop {
                let start_time = std::time::Instant::now();
                {
                    let _guard = lock.lock().await;
                    Logger::info(Category::JobManager, format!("Starting {}...", job_name).as_str());
                    job_fn(handler.clone(), notify_on_job_error, discord_webhook_url.clone()).await;
                }
                let elapsed = start_time.elapsed();
                let sleep_duration = Duration::from_hours(interval_hours as u64).saturating_sub(elapsed);
                Logger::info(Category::JobManager, format!("{} finished, next run in {} hours", job_name, interval_hours).as_str());
                sleep(sleep_duration).await;
            }
        });
    }

    pub async fn wait_for_jobs_to_finish(&self) {
        match self.job_lock.try_lock() {
            Ok(_) => {
                Logger::info(Category::JobManager, "No jobs are running");
            }
            Err(_) => {
                Logger::warn(Category::JobManager, "A job is still running, waiting for it to finish...");
                let _ = self.job_lock.lock().await;
                Logger::info(Category::JobManager, "All jobs finished");
            }
        }
    }
}
