pub enum TrackerStatus {
    Disabled,     // 0
    NotContacted, // 1
    Working,      // 2
    Updating,     // 3
    NotWorking,   // 4
}

impl TrackerStatus {
    pub fn from_int(num: i8) -> Result<TrackerStatus, String> {
        match num {
            0 => Ok(TrackerStatus::Disabled),
            1 => Ok(TrackerStatus::NotContacted),
            2 => Ok(TrackerStatus::Working),
            3 => Ok(TrackerStatus::Updating),
            4 => Ok(TrackerStatus::NotWorking),
            _ => Err(format!("Invalid Tracker Status Number: {}", num)),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            TrackerStatus::Disabled => String::from("Disabled"),
            TrackerStatus::NotContacted => String::from("Not Contacted"),
            TrackerStatus::Working => String::from("Working"),
            TrackerStatus::Updating => String::from("Updating"),
            TrackerStatus::NotWorking => String::from("Not Working"),
        }
    }
}
