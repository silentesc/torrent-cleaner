pub enum Category {
    Qbittorrent,
    Setup,
    JobManager,
    DiscordNotifier,
    Striker,
    HandleUnlinked,
    HandleNotWorking,
    HandleOrphaned,
}

impl Category {
    pub fn to_string(&self) -> String {
        match self {
            Category::Qbittorrent => String::from("qbittorrent"),
            Category::Setup => String::from("setup"),
            Category::JobManager => String::from("job_manager"),
            Category::DiscordNotifier => String::from("discord_notifier"),
            Category::Striker => String::from("striker"),
            Category::HandleUnlinked => String::from("handle_unlinked"),
            Category::HandleNotWorking => String::from("handle_not_working"),
            Category::HandleOrphaned => String::from("handle_orphaned"),
        }
    }
}
