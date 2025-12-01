use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Notification {
    discord_webhook_url: String,
}

impl Notification {
    pub fn discord_webhook_url(&self) -> &str {
        &self.discord_webhook_url
    }
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct HandleForgotten {
    interval_hours: i32,
    min_seeding_days: i32,
    min_strike_days: i32,
    required_strikes: i32,
    protection_tag: String,
    action: String,
}

impl HandleForgotten {
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

#[derive(Serialize, Deserialize)]
pub struct HandleNotWorking {
    interval_hours: i32,
    min_strike_days: i32,
    required_strikes: i32,
    protection_tag: String,
    action: String,
}

impl HandleNotWorking {
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
    pub fn action(&self) -> &str {
        &self.action
    }
}

#[derive(Serialize, Deserialize)]
pub struct HandleOrphaned {
    interval_hours: i32,
    min_strike_days: i32,
    required_strikes: i32,
    protection_tag: String,
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
    pub fn protection_tag(&self) -> &str {
        &self.protection_tag
    }
    pub fn action(&self) -> &str {
        &self.action
    }
}

#[derive(Serialize, Deserialize)]
pub struct Jobs {
    handle_forgotten: HandleForgotten,
    handle_not_working: HandleNotWorking,
    handle_orphaned: HandleOrphaned,
}

impl Jobs {
    pub fn handle_forgotten(&self) -> &HandleForgotten {
        &self.handle_forgotten
    }
    pub fn handle_not_working(&self) -> &HandleNotWorking {
        &self.handle_not_working
    }
    pub fn handle_orphaned(&self) -> &HandleOrphaned {
        &self.handle_orphaned
    }
}

#[derive(Serialize, Deserialize)]
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
            },
            torrent_client: TorrentClient {
                client: String::from(""),
                base_url: String::from(""),
                username: String::from(""),
                password: String::from(""),
            },
            jobs: Jobs {
                handle_forgotten: HandleForgotten {
                    interval_hours: 24,
                    min_seeding_days: 20,
                    min_strike_days: 3,
                    required_strikes: 3,
                    protection_tag: String::from("protected"),
                    action: String::from("test"),
                },
                handle_not_working: HandleNotWorking {
                    interval_hours: 3,
                    min_strike_days: 5,
                    required_strikes: 10,
                    protection_tag: String::from("protected"),
                    action: String::from("test"),
                },
                handle_orphaned: HandleOrphaned {
                    interval_hours: 24,
                    min_strike_days: 3,
                    required_strikes: 3,
                    protection_tag: String::from("protected"),
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
