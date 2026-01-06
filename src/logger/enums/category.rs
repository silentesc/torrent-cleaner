use std::fmt;

pub enum Category {
    Qbittorrent,
    Setup,
    JobManager,
    DiscordNotifier,
    Striker,
    FileUtils,
    DbManager,
    HandleUnlinked,
    HandleNotWorking,
    HandleOrphaned,
    HealthCheck,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let category_str = match self {
            Category::Qbittorrent => String::from("qbittorrent"),
            Category::Setup => String::from("setup"),
            Category::JobManager => String::from("job_manager"),
            Category::DiscordNotifier => String::from("discord_notifier"),
            Category::Striker => String::from("striker"),
            Category::FileUtils => String::from("file_utils"),
            Category::DbManager => String::from("db_manager"),
            Category::HandleUnlinked => String::from("handle_unlinked"),
            Category::HandleNotWorking => String::from("handle_not_working"),
            Category::HandleOrphaned => String::from("handle_orphaned"),
            Category::HealthCheck => String::from("health_check"),
        };
        write!(f, "{}", category_str)
    }
}
