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
    HandleUnregistered,
    HandleOrphaned,
    HealthCheckFiles,
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
            Category::HandleUnregistered => String::from("handle_unregistered"),
            Category::HandleOrphaned => String::from("handle_orphaned"),
            Category::HealthCheckFiles => String::from("health_check_files"),
        };
        write!(f, "{}", category_str)
    }
}
