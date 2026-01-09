use std::{sync::Arc, time::Duration};

use anyhow::Context;
use chrono::{NaiveDateTime, TimeDelta};
use reqwest::Url;
use rusqlite::{OptionalExtension, params};
use tokio::{sync::Mutex, time::sleep};

use crate::{
    config::Config,
    error, info,
    jobs::{handle_orphaned::runner::HandleOrphaned, handle_unlinked::runner::HandleUnlinked, handle_unregistered::runner::HandleUnregistered, health_check_files::runner::HealthCheckFiles},
    logger::enums::category::Category,
    torrent_clients::torrent_manager::TorrentManager,
    utils::{date_utils::DateUtils, db_manager::Session, discord_webhook_utils::DiscordWebhookUtils},
    warn,
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
        let handle_unregistered = Arc::new(HandleUnregistered::new(self.torrent_manager.clone(), self.config.clone()));
        let handle_orphaned = Arc::new(HandleOrphaned::new(self.torrent_manager.clone(), self.config.clone(), self.torrents_path.clone()));
        let health_check_files = Arc::new(HealthCheckFiles::new(self.torrent_manager.clone(), self.config.clone()));

        let discord_webhook_url = Some(self.config.notification().discord_webhook_url()).filter(|s| !s.is_empty()).and_then(|url_str| Url::parse(url_str).ok());

        self.spawn_job(
            String::from("handle_unlinked"),
            self.config.jobs().handle_unlinked().interval_hours(),
            Config::default().jobs().handle_unlinked().interval_hours(),
            *self.config.notification().on_job_error(),
            discord_webhook_url.clone(),
            handle_unlinked.clone(),
            |handler: Arc<HandleUnlinked>| async move { handler.run().await },
        );

        self.spawn_job(
            String::from("handle_unregistered"),
            self.config.jobs().handle_unregistered().interval_hours(),
            Config::default().jobs().handle_unregistered().interval_hours(),
            *self.config.notification().on_job_error(),
            discord_webhook_url.clone(),
            handle_unregistered.clone(),
            |handler: Arc<HandleUnregistered>| async move { handler.run().await },
        );

        self.spawn_job(
            String::from("handle_orphaned"),
            self.config.jobs().handle_orphaned().interval_hours(),
            Config::default().jobs().handle_orphaned().interval_hours(),
            *self.config.notification().on_job_error(),
            discord_webhook_url.clone(),
            handle_orphaned.clone(),
            |handler: Arc<HandleOrphaned>| async move { handler.run().await },
        );

        self.spawn_job(
            String::from("health_check_files"),
            self.config.jobs().health_check_files().interval_hours(),
            Config::default().jobs().health_check_files().interval_hours(),
            *self.config.notification().on_job_error(),
            discord_webhook_url.clone(),
            health_check_files.clone(),
            |handler: Arc<HealthCheckFiles>| async move { handler.run().await },
        );
    }

    fn spawn_job<T, F, Fut>(&self, job_name: String, mut interval_hours: i32, default_interval_hours: i32, notify_on_job_error: bool, discord_webhook_url: Option<Url>, handler: Arc<T>, job_fn: F)
    where
        T: Send + Sync + 'static,
        F: (Fn(Arc<T>) -> Fut) + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), anyhow::Error>> + Send + 'static,
    {
        if interval_hours == -1 {
            return;
        }

        let lock = self.job_lock.clone();

        tokio::spawn(async move {
            loop {
                ///////////
                // Sleep //
                ///////////
                if interval_hours == 0 {
                    interval_hours = default_interval_hours;
                } else {
                    let last_run = JobManager::get_last_run(job_name.as_str()).unwrap_or(None).unwrap_or_default();
                    let sleep_minutes = JobManager::get_startup_sleep_minutes(job_name.as_str(), interval_hours as i64);

                    info!(
                        Category::JobManager,
                        "{} (interval of {}h) last ran {:.2}h ago, next run in {:.2}h",
                        job_name,
                        interval_hours,
                        (DateUtils::get_current_local_naive_datetime() - last_run).num_minutes() as f64 / 60.0,
                        sleep_minutes as f64 / 60.0
                    );

                    // If interval_hours is 0 use default instead
                    interval_hours = match interval_hours {
                        0 => default_interval_hours,
                        _ => interval_hours,
                    };

                    sleep(Duration::from_mins(sleep_minutes as u64)).await;
                }

                /////////////
                // Run job //
                /////////////
                {
                    let _guard = lock.lock().await;

                    info!(Category::JobManager, "Starting {}...", job_name);

                    // Set last job run, then run job and save result
                    let result: Result<(), anyhow::Error> = async {
                        JobManager::set_last_run(job_name.as_str())?;
                        job_fn(handler.clone()).await?;
                        Ok(())
                    }
                    .await;

                    // Check result for error and log & send discord message
                    if let Err(e) = result {
                        error!(Category::JobManager, "Failed to run {}: {:#}", job_name, e);
                        // Notify on discord
                        if notify_on_job_error {
                            let mut discord_webhook_utils = DiscordWebhookUtils::new(discord_webhook_url.clone());
                            if let Err(e) = discord_webhook_utils
                                .send_webhook_embed("Error", format!("`{}` threw an error. Please check logs for more details.", job_name).as_str(), vec![])
                                .await
                            {
                                error!(Category::JobManager, "Error while sending discord webhook error message: {:#}", e);
                            }
                        }
                    }
                }
            }
        });
    }

    pub async fn wait_for_jobs_to_finish(&self) {
        match self.job_lock.try_lock() {
            Ok(_) => {
                info!(Category::JobManager, "No jobs are running");
            }
            Err(_) => {
                warn!(Category::JobManager, "A job is still running, waiting for it to finish...");
                let _ = self.job_lock.lock().await;
                info!(Category::JobManager, "All jobs finished");
            }
        }
    }

    ///////////////////
    // Private Utils //
    ///////////////////

    fn get_startup_sleep_minutes(job_name: &str, interval_hours: i64) -> i64 {
        match JobManager::get_last_run(job_name) {
            // Check for get_last_run error
            Ok(last_run_naive_datetime_option) => match last_run_naive_datetime_option {
                // Check for None
                Some(last_run_naive_datetime) => {
                    // If the last job run is bigger than interval_hours then it's long ago and should instantly run
                    // Else the job would wait longer than it has to, so return the interval minus the time since the last run
                    let time_delta = DateUtils::get_current_local_naive_datetime() - last_run_naive_datetime;
                    if time_delta > TimeDelta::hours(interval_hours) { 0 } else { interval_hours * 60 - time_delta.num_minutes() }
                }
                None => interval_hours * 60,
            },
            Err(e) => {
                error!(Category::JobManager, "Error while getting last job run for {}: {:#}", job_name, e);
                interval_hours * 60
            }
        }
    }

    //////////////
    // Db Stuff //
    //////////////

    fn get_last_run(job_name: &str) -> Result<Option<NaiveDateTime>, anyhow::Error> {
        let session = Session::new()?;
        let conn = session.into_conn().ok_or(anyhow::anyhow!("Failed to get conn from session"))?;

        let mut stmt = conn.prepare("SELECT last_job_run FROM jobs WHERE job_name = ?1").context("Failed to prepare get last job run")?;
        let last_job_run_str_option: Option<String> = stmt.query_one(params![job_name], |row| row.get(0)).optional().context("Failed to query last job run")?;

        match last_job_run_str_option {
            Some(last_job_run_str) => {
                let last_job_run = DateUtils::parse_naive_datetime_from_str(last_job_run_str.as_str())?;
                Ok(Some(last_job_run))
            }
            None => Ok(None),
        }
    }

    fn set_last_run(job_name: &str) -> Result<(), anyhow::Error> {
        let session = Session::new()?;
        let conn = session.into_conn().ok_or(anyhow::anyhow!("Failed to get conn from session"))?;

        conn.execute(
            "INSERT OR REPLACE INTO jobs(job_name, last_job_run) VALUES(?1, ?2)",
            params![job_name, DateUtils::convert_naive_datetime_to_string(DateUtils::get_current_local_naive_datetime())],
        )
        .context("Failed to insert or update last job run")?;

        Ok(())
    }
}
