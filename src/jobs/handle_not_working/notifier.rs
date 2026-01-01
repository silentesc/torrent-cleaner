use chrono::{Local, TimeZone};

use crate::{
    config::Config,
    jobs::utils::discord_webhook_utils::{DiscordWebhookUtils, EmbedField},
    logger::{enums::category::Category},
    torrent_clients::{
        enums::tracker_status::TrackerStatus,
        models::{torrent::Torrent, tracker::Tracker},
    }, warn,
};

pub struct Notifier;

impl Notifier {
    /**
     * Send notification
     */
    pub async fn send_notification(discord_webhook_utils: &mut DiscordWebhookUtils, torrent: &Torrent, trackers: &Vec<Tracker>, config: &Config) -> Result<(), anyhow::Error> {
        if !discord_webhook_utils.is_notifications_enabled() {
            return Ok(());
        }

        let total_size_gib = format!("{:.2}", (*torrent.total_size() / 1024 / 1024) as f32 / 1024.0);
        let total_size_gb = format!("{:.2}", (*torrent.total_size() / 1000 / 1000) as f32 / 1000.0);

        let seeding_days = format!("{:.2}", (*torrent.seeding_time() / 60 / 60) as f32 / 24.0);

        let added_on_str = match Local.timestamp_opt(*torrent.added_on(), 0).single() {
            Some(datetime_local) => datetime_local.format("%Y-%m-%d %H:%M:%S").to_string(),
            None => String::from("Failed getting datetime"),
        };
        let completed_on_str = match *torrent.completion_on() {
            -1 => String::from("Not completed"),
            _ => match Local.timestamp_opt(*torrent.completion_on(), 0).single() {
                Some(datetime_local) => datetime_local.format("%Y-%m-%d %H:%M:%S").to_string(),
                None => String::from("Failed getting datetime"),
            },
        };

        let mut fields: Vec<EmbedField> = Vec::new();
        for tracker in trackers {
            let tracker_status_str = match TrackerStatus::from_int(*tracker.status()) {
                Ok(tracker_status) => tracker_status.to_string(),
                Err(e) => {
                    warn!(Category::HandleNotWorking, "{}", e);
                    tracker.status().to_string()
                }
            };
            fields.push(EmbedField {
                name: String::from("Tracker"),
                value: format!("URL: {}\nStatus: {}\nMessage: {}", tracker.url(), tracker_status_str, tracker.msg()),
                inline: false,
            });
        }
        fields.extend(vec![
            EmbedField {
                name: String::from("Action"),
                value: config.jobs().handle_not_working().action().to_string(),
                inline: true,
            },
            EmbedField {
                name: String::from("Category"),
                value: torrent.category().to_string(),
                inline: true,
            },
            EmbedField {
                name: String::from("Tags"),
                value: torrent.tags().to_string(),
                inline: true,
            },
            EmbedField {
                name: String::from("Total Size"),
                value: format!("{total_size_gib}GiB | {total_size_gb}GB"),
                inline: true,
            },
            EmbedField {
                name: String::from("Ratio"),
                value: format!("{:.2}", torrent.ratio()),
                inline: true,
            },
            EmbedField {
                name: String::from("Seeding days"),
                value: seeding_days.to_string(),
                inline: true,
            },
            EmbedField {
                name: String::from("Added"),
                value: added_on_str,
                inline: true,
            },
            EmbedField {
                name: String::from("Completed"),
                value: completed_on_str,
                inline: true,
            },
        ]);

        discord_webhook_utils.send_webhook_embed(torrent.name(), "Found not working torrent", fields).await
    }
}
