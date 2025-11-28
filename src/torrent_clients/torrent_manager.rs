use std::sync::Arc;

use crate::torrent_clients::{models::torrent::Torrent, traits::torrent_client::TorrentClient};

pub struct TorrentManager<C: TorrentClient> {
    torrent_client: Arc<C>,
}

impl<C: TorrentClient> TorrentManager<C> {
    pub fn new(torrent_client: Arc<C>) -> Self {
        Self { torrent_client }
    }

    pub async fn login(&self) -> Result<(), anyhow::Error> {
        self.torrent_client.login().await
    }

    pub async fn logout(&self) -> Result<(), anyhow::Error> {
        self.torrent_client.logout().await
    }

    pub async fn get_all_torrents(&self) -> Result<Vec<Torrent<C>>, anyhow::Error> {
        self.torrent_client.get_all_torrents().await
    }
}
