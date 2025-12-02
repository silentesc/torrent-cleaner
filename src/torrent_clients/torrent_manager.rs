use crate::torrent_clients::{
    enums::any_client::AnyClient,
    models::{torrent::Torrent, tracker::Tracker},
    traits::torrent_client::TorrentClient,
};

pub struct TorrentManager {
    torrent_client: AnyClient,
}

impl TorrentManager {
    pub fn new(torrent_client: AnyClient) -> Self {
        Self { torrent_client }
    }

    pub async fn login(&self) -> Result<(), anyhow::Error> {
        self.torrent_client.login().await
    }

    pub async fn logout(&self) -> Result<(), anyhow::Error> {
        self.torrent_client.logout().await
    }

    pub async fn get_all_torrents(&self) -> Result<Vec<Torrent>, anyhow::Error> {
        self.torrent_client.get_all_torrents().await
    }

    pub async fn get_torrent_trackers(&self, torrent_hash: &str) -> Result<Vec<Tracker>, anyhow::Error> {
        self.torrent_client.get_torrent_trackers(torrent_hash).await
    }

    pub async fn stop_torrent(&self, torrent_hash: &str) -> Result<(), anyhow::Error> {
        self.torrent_client.stop_torrent(torrent_hash).await
    }

    pub async fn delete_torrent(&self, torrent_hash: &str, delete_files: bool) -> Result<(), anyhow::Error> {
        self.torrent_client.delete_torrent(torrent_hash, delete_files).await
    }
}
