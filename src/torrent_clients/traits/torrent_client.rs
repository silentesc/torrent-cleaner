use crate::torrent_clients::models::{torrent::Torrent, tracker::Tracker};

pub trait TorrentClient {
    async fn login(&self) -> Result<(), anyhow::Error>;
    async fn logout(&self) -> Result<(), anyhow::Error>;
    async fn get_all_torrents(&self) -> Result<Vec<Torrent>, anyhow::Error>;
    async fn get_torrent_trackers(&self, torrent_hash: &str) -> Result<Vec<Tracker>, anyhow::Error>;
    async fn stop_torrent(&self, torrent_hash: &str) -> Result<(), anyhow::Error>;
    async fn delete_torrent(&self, torrent_hash: &str, delete_files: bool) -> Result<(), anyhow::Error>;
}
