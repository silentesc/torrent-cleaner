use crate::torrent_clients::{
    adapters::qbittorrent::Qbittorrent,
    models::{torrent::Torrent, torrent_file::TorrentFile, tracker::Tracker},
    traits::torrent_client::TorrentClient,
};

pub enum AnyClient {
    Qbittorrent(Qbittorrent),
}

impl TorrentClient for AnyClient {
    async fn login(&self) -> Result<(), anyhow::Error> {
        match self {
            AnyClient::Qbittorrent(c) => c.login().await,
        }
    }

    async fn logout(&self) -> Result<(), anyhow::Error> {
        match self {
            AnyClient::Qbittorrent(c) => c.logout().await,
        }
    }

    async fn is_logged_in(&self) -> Result<bool, anyhow::Error> {
        match self {
            AnyClient::Qbittorrent(c) => c.is_logged_in().await,
        }
    }

    async fn get_all_torrents(&self) -> Result<Vec<Torrent>, anyhow::Error> {
        match self {
            AnyClient::Qbittorrent(c) => c.get_all_torrents().await,
        }
    }

    async fn get_torrent_trackers(&self, torrent_hash: &str) -> Result<Vec<Tracker>, anyhow::Error> {
        match self {
            AnyClient::Qbittorrent(c) => c.get_torrent_trackers(torrent_hash).await,
        }
    }

    async fn get_torrent_files(&self, torrent_hash: &str) -> Result<Vec<TorrentFile>, anyhow::Error> {
        match self {
            AnyClient::Qbittorrent(c) => c.get_torrent_files(torrent_hash).await,
        }
    }

    async fn stop_torrent(&self, torrent_hash: &str) -> Result<(), anyhow::Error> {
        match self {
            AnyClient::Qbittorrent(c) => c.stop_torrent(torrent_hash).await,
        }
    }

    async fn delete_torrent(&self, torrent_hash: &str, delete_files: bool) -> Result<(), anyhow::Error> {
        match self {
            AnyClient::Qbittorrent(c) => c.delete_torrent(torrent_hash, delete_files).await,
        }
    }
}
