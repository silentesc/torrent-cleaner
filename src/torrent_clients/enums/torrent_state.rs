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
    pub fn as_str(&self) -> &str {
        match self {
            TorrentState::Error => "error",
            TorrentState::MissingFiles => "missingFiles",
            TorrentState::Uploading => "uploading",
            TorrentState::PausedUP => "pausedUP",
            TorrentState::QueuedUP => "queuedUP",
            TorrentState::StalledUP => "stalledUP",
            TorrentState::CheckingUP => "checkingUP",
            TorrentState::ForcedUP => "forcedUP",
            TorrentState::Allocating => "allocating",
            TorrentState::Downloading => "downloading",
            TorrentState::MetaDL => "metaDL",
            TorrentState::PausedDL => "pausedDL",
            TorrentState::QueuedDL => "queuedDL",
            TorrentState::StalledDL => "stalledDL",
            TorrentState::CheckingDL => "checkingDL",
            TorrentState::ForcedDL => "forcedDL",
            TorrentState::CheckingResumeData => "checkingResumeData",
            TorrentState::Moving => "moving",
            TorrentState::Unknown => "unknown",
        }
    }
}
