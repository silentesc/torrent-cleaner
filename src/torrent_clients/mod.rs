pub mod torrent_manager;
pub mod adapters {
    pub mod qbittorrent;
}
pub mod enums {
    pub mod any_client;
    pub mod torrent_state;
    pub mod tracker_status;
}
pub mod models {
    pub mod torrent;
    pub mod torrent_file;
    pub mod tracker;
}
pub mod traits {
    pub mod torrent_client;
}
