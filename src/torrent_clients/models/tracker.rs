use serde::Deserialize;

use crate::torrent_clients::enums::tracker_status::TrackerStatus;

static UNREGISTERED_MESSAGES: [&str; 33] = [
    "complete season uploaded",
    "dead",
    "dupe",
    "i'm sorry dave, i can't do that",
    "infohash not found",
    "internal available",
    "not exist",
    "not registered",
    "nuked",
    "pack is available",
    "packs are available",
    "problem with description",
    "problem with file",
    "problem with pack",
    "retitled",
    "season pack",
    "specifically banned",
    "torrent does not exist",
    "torrent existiert nicht",
    "torrent has been deleted",
    "torrent has been nuked",
    "torrent introuvable",
    "torrent is not authorized for use on this tracker",
    "torrent is not found",
    "torrent nicht gefunden",
    "tracker nicht registriert",
    "torrent not found",
    "trump",
    "unknown",
    "unregistered",
    "não registrado",
    "upgraded",
    "uploaded",
];

#[derive(Deserialize)]
pub struct Tracker {
    url: String,
    status: i8,
    msg: String,
}

impl Tracker {
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn status(&self) -> &i8 {
        &self.status
    }
    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn is_unregistered(&self) -> bool {
        let mut is_msg_unregistered = false;

        for unregistered_msg in UNREGISTERED_MESSAGES {
            if self.msg.to_lowercase().contains(unregistered_msg) {
                is_msg_unregistered = true;
                break;
            }
        }

        self.status != TrackerStatus::Working.to_i8() && is_msg_unregistered
    }
}
