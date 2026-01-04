use std::fmt;

pub enum TrackerStatus {
    Disabled,     // 0
    NotContacted, // 1
    Working,      // 2
    Updating,     // 3
    NotWorking,   // 4
}

impl fmt::Display for TrackerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            TrackerStatus::Disabled => String::from("Disabled"),
            TrackerStatus::NotContacted => String::from("Not Contacted"),
            TrackerStatus::Working => String::from("Working"),
            TrackerStatus::Updating => String::from("Updating"),
            TrackerStatus::NotWorking => String::from("Not Working"),
        };
        write!(f, "{}", status_str)
    }
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
}
