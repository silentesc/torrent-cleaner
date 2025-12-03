use serde::Deserialize;

use crate::torrent_clients::enums::tracker_status::TrackerStatus;

#[derive(Deserialize)]
pub struct Tracker {
    url: String,
    status: i8,
    msg: String,
}

impl Tracker {
    pub fn println(&self) {
        let status_str = match TrackerStatus::from_int(self.status) {
            Ok(tracker_status) => tracker_status.as_str().to_string(),
            Err(error_msg) => error_msg.to_string(),
        };
        println!("Tracker {}", self.url);
        println!("  status:    {}", status_str);
        println!("  msg:       {}", self.msg);
    }
}
