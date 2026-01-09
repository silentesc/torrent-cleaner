use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Notification {
    discord_webhook_url: String,
    on_job_action: bool,
    on_job_error: bool,
}

impl Notification {
    pub fn discord_webhook_url(&self) -> &str {
        &self.discord_webhook_url
    }

    pub fn on_job_action(&self) -> &bool {
        &self.on_job_action
    }

    pub fn on_job_error(&self) -> &bool {
        &self.on_job_error
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TorrentClient {
    client: String,
    base_url: String,
    username: String,
    password: String,
}

impl TorrentClient {
    pub fn client(&self) -> &str {
        &self.client
    }
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn password(&self) -> &str {
        &self.password
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HandleUnlinked {
    interval_hours: i32,
    min_seeding_days: i32,
    min_strike_days: i32,
    required_strikes: i32,
    protection_tag: String,
    action: String,
}

impl HandleUnlinked {
    pub fn interval_hours(&self) -> i32 {
        self.interval_hours
    }
    pub fn min_seeding_days(&self) -> i32 {
        self.min_seeding_days
    }
    pub fn min_strike_days(&self) -> i32 {
        self.min_strike_days
    }
    pub fn required_strikes(&self) -> i32 {
        self.required_strikes
    }
    pub fn protection_tag(&self) -> &str {
        &self.protection_tag
    }
    pub fn action(&self) -> &str {
        &self.action
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HandleUnregistered {
    interval_hours: i32,
    min_strike_days: i32,
    required_strikes: i32,
    protection_tag: String,
    ignore_dht: bool,
    ignore_pex: bool,
    ignore_lsd: bool,
    action: String,
}

impl HandleUnregistered {
    pub fn interval_hours(&self) -> i32 {
        self.interval_hours
    }
    pub fn min_strike_days(&self) -> i32 {
        self.min_strike_days
    }
    pub fn required_strikes(&self) -> i32 {
        self.required_strikes
    }
    pub fn protection_tag(&self) -> &str {
        &self.protection_tag
    }
    pub fn ignore_dht(&self) -> &bool {
        &self.ignore_dht
    }
    pub fn ignore_pex(&self) -> &bool {
        &self.ignore_pex
    }
    pub fn ignore_lsd(&self) -> &bool {
        &self.ignore_lsd
    }
    pub fn action(&self) -> &str {
        &self.action
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HandleOrphaned {
    interval_hours: i32,
    min_strike_days: i32,
    required_strikes: i32,
    protect_external_hardlinks: bool,
    action: String,
}

impl HandleOrphaned {
    pub fn interval_hours(&self) -> i32 {
        self.interval_hours
    }
    pub fn min_strike_days(&self) -> i32 {
        self.min_strike_days
    }
    pub fn required_strikes(&self) -> i32 {
        self.required_strikes
    }
    pub fn protect_external_hardlinks(&self) -> &bool {
        &self.protect_external_hardlinks
    }
    pub fn action(&self) -> &str {
        &self.action
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HealthCheckFiles {
    interval_hours: i32,
    action: String,
}

impl HealthCheckFiles {
    pub fn interval_hours(&self) -> i32 {
        self.interval_hours
    }
    pub fn action(&self) -> &str {
        &self.action
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Jobs {
    handle_unlinked: HandleUnlinked,
    handle_unregistered: HandleUnregistered,
    handle_orphaned: HandleOrphaned,
    health_check_files: HealthCheckFiles,
}

impl Jobs {
    pub fn handle_unlinked(&self) -> &HandleUnlinked {
        &self.handle_unlinked
    }
    pub fn handle_unregistered(&self) -> &HandleUnregistered {
        &self.handle_unregistered
    }
    pub fn handle_orphaned(&self) -> &HandleOrphaned {
        &self.handle_orphaned
    }
    pub fn health_check_files(&self) -> &HealthCheckFiles {
        &self.health_check_files
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    notification: Notification,
    torrent_client: TorrentClient,
    jobs: Jobs,
}

impl Config {
    pub fn default() -> Self {
        Self {
            notification: Notification {
                discord_webhook_url: String::from(""),
                on_job_action: true,
                on_job_error: true,
            },
            torrent_client: TorrentClient {
                client: String::from(""),
                base_url: String::from(""),
                username: String::from(""),
                password: String::from(""),
            },
            jobs: Jobs {
                handle_unlinked: HandleUnlinked {
                    interval_hours: 12,
                    min_seeding_days: 20,
                    min_strike_days: 3,
                    required_strikes: 3,
                    protection_tag: String::from("protected-unlinked"),
                    action: String::from("test"),
                },
                handle_unregistered: HandleUnregistered {
                    interval_hours: 3,
                    min_strike_days: 1,
                    required_strikes: 2,
                    ignore_dht: true,
                    ignore_pex: true,
                    ignore_lsd: true,
                    protection_tag: String::from("protected-unregistered"),
                    action: String::from("test"),
                },
                handle_orphaned: HandleOrphaned {
                    interval_hours: 13,
                    min_strike_days: 3,
                    required_strikes: 3,
                    protect_external_hardlinks: true,
                    action: String::from("test"),
                },
                health_check_files: HealthCheckFiles {
                    interval_hours: 24,
                    action: String::from("test"),
                },
            },
        }
    }

    pub fn notification(&self) -> &Notification {
        &self.notification
    }
    pub fn torrent_client(&self) -> &TorrentClient {
        &self.torrent_client
    }
    pub fn jobs(&self) -> &Jobs {
        &self.jobs
    }
}
