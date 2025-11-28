pub enum TrackerStatus {
    Disabled,     // 0
    NotContacted, // 1
    Working,      // 2
    Updating,     // 3
    NotWorking,   // 4
}

impl TrackerStatus {
    pub fn from_int(num: i8) -> Result<TrackerStatus, &'static str> {
        match num {
            0 => Ok(TrackerStatus::Disabled),
            1 => Ok(TrackerStatus::NotContacted),
            2 => Ok(TrackerStatus::Working),
            3 => Ok(TrackerStatus::Updating),
            4 => Ok(TrackerStatus::NotWorking),
            _ => Err("Invalid Number"),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            TrackerStatus::Disabled => "Disabled",
            TrackerStatus::NotContacted => "NotContacted",
            TrackerStatus::Working => "Working",
            TrackerStatus::Updating => "Updating",
            TrackerStatus::NotWorking => "NotWorking",
        }
    }
}
