pub enum TorrentState {
    Error,
    MissingFiles,
    Uploading,
    PausedUP,
    QueuedUP,
    StalledUP,
    CheckingUP,
    ForcedUP,
    Allocating,
    Downloading,
    MetaDL,
    PausedDL,
    QueuedDL,
    StalledDL,
    CheckingDL,
    ForcedDL,
    CheckingResumeData,
    Moving,
    Unknown,
}

impl TorrentState {
    pub fn as_string(&self) -> String {
        match self {
            TorrentState::Error => String::from("error"),
            TorrentState::MissingFiles => String::from("missingFiles"),
            TorrentState::Uploading => String::from("uploading"),
            TorrentState::PausedUP => String::from("pausedUP"),
            TorrentState::QueuedUP => String::from("queuedUP"),
            TorrentState::StalledUP => String::from("stalledUP"),
            TorrentState::CheckingUP => String::from("checkingUP"),
            TorrentState::ForcedUP => String::from("forcedUP"),
            TorrentState::Allocating => String::from("allocating"),
            TorrentState::Downloading => String::from("downloading"),
            TorrentState::MetaDL => String::from("metaDL"),
            TorrentState::PausedDL => String::from("pausedDL"),
            TorrentState::QueuedDL => String::from("queuedDL"),
            TorrentState::StalledDL => String::from("stalledDL"),
            TorrentState::CheckingDL => String::from("checkingDL"),
            TorrentState::ForcedDL => String::from("forcedDL"),
            TorrentState::CheckingResumeData => String::from("checkingResumeData"),
            TorrentState::Moving => String::from("moving"),
            TorrentState::Unknown => String::from("unknown"),
        }
    }
}
