use std::sync::Arc;

use serde::Deserialize;

use crate::torrent_clients::{models::tracker::Tracker, traits::torrent_client::TorrentClient};

#[derive(Deserialize, Clone)]
pub struct TorrentInfo {
    pub hash: String,
    pub name: String,
    pub total_size: i64,
    pub content_path: String,
    pub ratio: f32,
    pub state: String,
    pub tracker: String,
    pub category: String,
    pub tags: String,
    pub added_on: i64,
    pub completion_on: i64,
    pub seeding_time: i64,
}

#[derive(Clone)]
pub struct Torrent<C: TorrentClient + Clone> {
    info: TorrentInfo,
    torrent_client: Arc<C>,
}

impl<C: TorrentClient + Clone> Torrent<C> {
    pub fn new(info: TorrentInfo, torrent_client: Arc<C>) -> Self {
        Torrent { info, torrent_client }
    }

    pub fn get_info(&self) -> TorrentInfo {
        return self.info.clone();
    }

    pub async fn get_trackers(&self) -> Result<Vec<Tracker>, anyhow::Error> {
        self.torrent_client.get_torrent_trackers(&self.info.hash).await
    }

    pub async fn stop(&self) -> Result<(), anyhow::Error> {
        self.torrent_client.stop_torrent(&self.info.hash).await
    }

    pub async fn delete(&self, delete_files: bool) -> Result<(), anyhow::Error> {
        self.torrent_client.delete_torrent(&self.info.hash, delete_files).await
    }

    pub fn println(&self) {
        println!("Torrent {}", self.info.hash);
        println!("  name:          {}", self.info.name);
        println!("  total_size:    {}", self.info.total_size);
        println!("  content_path:  {}", self.info.content_path);
        println!("  ratio:         {}", self.info.ratio);
        println!("  state:         {}", self.info.state);
        println!("  tracker:       {}", self.info.tracker);
        println!("  category:      {}", self.info.category);
        println!("  tags:          {}", self.info.tags);
        println!("  added_on:      {}", self.info.added_on);
        println!("  completion_on: {}", self.info.completion_on);
        println!("  seeding_time:  {}", self.info.seeding_time);
    }
}
