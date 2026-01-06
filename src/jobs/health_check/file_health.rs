use std::{os::unix::fs::MetadataExt, path::Path, sync::Arc};

use anyhow::Context;

use crate::{
    debug,
    logger::enums::category::Category,
    torrent_clients::{models::torrent::Torrent, torrent_manager::TorrentManager},
};

pub struct FileHealth;

impl FileHealth {
    pub async fn check_files(torrent_manager: Arc<TorrentManager>, torrents: &Vec<Torrent>) -> Result<Vec<String>, anyhow::Error> {
        let mut issues: Vec<String> = Vec::new();

        for torrent in torrents {
            if *torrent.completion_on() == -1 {
                debug!(Category::HealthCheck, "Torrent not completed: ({}) {}", torrent.hash(), torrent.name());
                continue;
            }
            let torrent_files = torrent_manager.get_torrent_files(torrent.hash()).await.context("Getting torrent file failed")?;
            for torrent_file in torrent_files {
                let path_str = format!("{}/{}", torrent.save_path(), torrent_file.name());
                let path = Path::new(&path_str);

                // Check exist
                if !path.try_exists().context(format!("Failed to check if file exists: {}", path_str))? {
                    issues.push(format!("File for torrent ({}) {} does not exist: {}", torrent.name(), torrent.hash(), path_str));
                    return Ok(issues);
                }

                let metadata = Path::new(&path_str).metadata().context(format!("Failed getting metadata of {}", path_str))?;

                // Check size
                if *torrent_file.size() != metadata.size() {
                    issues.push(format!(
                        "Torrent file size ({} bytes) doesn't match actual file size ({} bytes) for torrent ({}) {}: {}",
                        torrent_file.size(),
                        metadata.size(),
                        torrent.name(),
                        torrent.hash(),
                        path_str
                    ));
                }

                // Check file is dir
                if metadata.file_type().is_dir() {
                    issues.push(format!("Torrent file is actually a directory for torrent {} ({}): {}", torrent.name(), torrent.hash(), path_str));
                }
            }
        }

        Ok(issues)
    }
}
