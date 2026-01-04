use std::fmt;

pub enum TorrentState {
    PausedUP,
    StoppedUP,
    PausedDL,
    StoppedDL,
}

impl fmt::Display for TorrentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state_str = match self {
            TorrentState::PausedUP => String::from("pausedUP"),
            TorrentState::StoppedUP => String::from("stoppedUP"),
            TorrentState::PausedDL => String::from("pausedDL"),
            TorrentState::StoppedDL => String::from("stoppedDL"),
        };
        write!(f, "{}", state_str)
    }
}
