pub enum Category {
    Qbittorrent,
    Setup,
    JobManager,
    DiscordNotifier,
    Striker,
    HandleForgotten,
    HandleNotWorking,
    HandleOrphaned,
    FileUtils,
}

impl Category {
    pub fn to_string(&self) -> String {
        match self {
            Category::Qbittorrent => String::from("qbittorrent"),
            Category::Setup => String::from("setup"),
            Category::JobManager => String::from("job_manager"),
            Category::DiscordNotifier => String::from("discord_notifier"),
            Category::Striker => String::from("striker"),
            Category::HandleForgotten => String::from("handle_forgotten"),
            Category::HandleNotWorking => String::from("handle_not_working"),
            Category::HandleOrphaned => String::from("handle_orphaned"),
            Category::FileUtils => String::from("file_utils"),
        }
    }
}
