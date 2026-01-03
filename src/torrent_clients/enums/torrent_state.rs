pub enum TorrentState {
    PausedUP,
    StoppedUP,
    PausedDL,
    StoppedDL,
}

impl TorrentState {
    pub fn to_string(&self) -> String {
        match self {
            TorrentState::PausedUP => String::from("pausedUP"),
            TorrentState::StoppedUP => String::from("stoppedUP"),
            TorrentState::PausedDL => String::from("pausedDL"),
            TorrentState::StoppedDL => String::from("stoppedDL"),
        }
    }
}
