use std::{collections::HashSet, fs, path::PathBuf, sync::Arc};

use anyhow::Context;
use walkdir::WalkDir;

use crate::{
    logger::{enums::category::Category, logger::Logger},
    torrent_clients::torrent_manager::TorrentManager,
};

pub struct Receiver;

impl Receiver {
    pub async fn get_orphaned_path_strings(torrent_paths: &HashSet<PathBuf>, torrents_path: &str) -> Result<HashSet<String>, anyhow::Error> {
        // Get paths not present in any torrents
        Logger::debug(Category::HandleOrphaned, "Getting orphaned paths (files/folders that are not part of any torrent)...");
        let mut orphaned_path_strings: HashSet<String> = HashSet::new();
        for entry in WalkDir::new(torrents_path) {
            let entry_result = entry.context("Failed to get entry_result")?;
            let path = entry_result.path();

            if torrent_paths.contains(path) {
                continue;
            }

            let file_type = entry_result.file_type();
            let mut is_orphan = false;

            // Check for file
            if file_type.is_file() {
                is_orphan = true;
            }
            // Check for empty dir
            else if file_type.is_dir() {
                let mut entries = fs::read_dir(path).context("Failed to read dir")?;
                if entries.next().is_none() {
                    is_orphan = true;
                }
            }
            // Handle edge case not file or dir (should not happen)
            else {
                return Err(anyhow::anyhow!("path is neither file or dir: {:?}", path));
            }

            if is_orphan {
                if let Some(path_str) = path.to_str() {
                    orphaned_path_strings.insert(path_str.to_string());
                } else {
                    return Err(anyhow::anyhow!("Failed to get string from path (may due to non-UTF8 path: {:?}", path));
                }
            }
        }
        Logger::debug(Category::HandleOrphaned, format!("Received {} orphaned paths", orphaned_path_strings.len()).as_str());

        Ok(orphaned_path_strings)
    }

    pub async fn get_torrent_paths(torrent_manager: Arc<TorrentManager>) -> Result<HashSet<PathBuf>, anyhow::Error> {
        // Get torrents from torrent client
        Logger::debug(Category::HandleOrphaned, "Getting torrents...");
        let torrents = torrent_manager.get_all_torrents().await.context("Failed to get all torrents")?;
        Logger::debug(Category::HandleOrphaned, format!("Received {} torrents", torrents.len()).as_str());

        // Get torrent paths
        Logger::debug(Category::HandleOrphaned, "Getting all paths in all torrents...");
        let mut torrent_paths: HashSet<PathBuf> = HashSet::new();
        for torrent in torrents {
            if torrent.content_path().is_empty() {
                Logger::warn(
                    Category::HandleOrphaned,
                    format!(
                        "Ignoring torrent with no content path (maybe due to torrent still checking metadata, missing/moving files, I/O errors, Permission errors): ({}) {}",
                        torrent.hash(),
                        torrent.name()
                    )
                    .as_str(),
                );
                continue;
            }
            for entry in WalkDir::new(torrent.content_path()) {
                let entry_result = entry.context("Failed to get entry_result")?;
                let path = entry_result.path();
                let file_type = entry_result.file_type();

                // Check for file
                if file_type.is_file() {
                    torrent_paths.insert(entry_result.into_path());
                }
                // Check for empty dir
                else if file_type.is_dir() {
                    let mut entries = fs::read_dir(path).context("Failed to read dir")?;
                    if entries.next().is_none() {
                        torrent_paths.insert(entry_result.into_path());
                    }
                }
                // Handle edge case not file or dir (should not happen)
                else {
                    return Err(anyhow::anyhow!("path is neither file or dir: {:?}", path));
                }
            }
        }
        Logger::debug(Category::HandleOrphaned, format!("Received {} unique torrent paths", torrent_paths.len()).as_str());

        Ok(torrent_paths)
    }
}
