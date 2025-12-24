use std::{collections::HashSet, fs, path::PathBuf, sync::Arc};

use anyhow::Context;
use walkdir::WalkDir;

use crate::{
    logger::{enums::category::Category, logger::Logger},
    torrent_clients::torrent_manager::TorrentManager,
};

pub struct Receiver;

impl Receiver {
    pub async fn get_orphaned_path_strings(torrent_paths: &HashSet<PathBuf>, torrents_path: &str) -> Result<Vec<String>, anyhow::Error> {
        // Get paths not present in any torrents
        Logger::debug(Category::HandleOrphaned, "Getting orphaned paths (files/folders that are not part of any torrent)...");
        let mut orphaned_path_strings: Vec<String> = Vec::new();
        for entry in WalkDir::new(torrents_path) {
            let entry_result = entry.context("Failed to get entry_result")?;
            // Check for file
            if entry_result.file_type().is_file() {
                let path_buf = entry_result.into_path();
                if torrent_paths.contains(&path_buf) {
                    continue;
                }
                let path_string = match path_buf.into_os_string().into_string() {
                    Ok(path_string) => path_string,
                    Err(os_string) => {
                        return Err(anyhow::anyhow!("Failed to convert PathBuf into string: {}", os_string.display()));
                    }
                };
                orphaned_path_strings.push(path_string);
            }
            // Check for empty dir
            else if entry_result.file_type().is_dir() {
                let path_buf = entry_result.clone().into_path();
                if torrent_paths.contains(&path_buf) {
                    continue;
                }
                let mut entries = fs::read_dir(entry_result.path()).context("Failed to read dir")?;
                if entries.next().is_some() {
                    continue;
                }
                let path_string = match path_buf.into_os_string().into_string() {
                    Ok(path_string) => path_string,
                    Err(os_string) => {
                        return Err(anyhow::anyhow!("Failed to convert PathBuf into string: {}", os_string.display()));
                    }
                };
                orphaned_path_strings.push(path_string);
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
            for entry in WalkDir::new(torrent.content_path()) {
                let entry_result = entry.context("Failed to get entry_result")?;
                // Check for file
                if entry_result.file_type().is_file() {
                    torrent_paths.insert(entry_result.into_path());
                }
                // Check for empty dir
                else if entry_result.file_type().is_dir() {
                    let mut entries = fs::read_dir(entry_result.path()).context("Failed to read dir")?;
                    if entries.next().is_none() {
                        torrent_paths.insert(entry_result.into_path());
                    }
                }
            }
        }
        Logger::debug(Category::HandleOrphaned, format!("Received {} unique torrent paths", torrent_paths.len()).as_str());

        Ok(torrent_paths)
    }
}
