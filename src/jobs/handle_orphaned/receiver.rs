use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use walkdir::WalkDir;

use crate::{
    jobs::utils::file_utils::FileUtils,
    logger::{enums::category::Category, logger::Logger},
    torrent_clients::torrent_manager::TorrentManager,
};

pub struct Receiver;

impl Receiver {
    pub async fn get_orphaned_path_strings(torrent_paths: &HashSet<PathBuf>, torrents_path: &str, protect_external_hardlinks: bool) -> Result<HashSet<String>, anyhow::Error> {
        // Get known hardlinks
        Logger::debug(Category::HandleOrphaned, "Getting known torrent hardlinks...");
        let known_hardlinks: HashMap<u64, u64> = FileUtils::get_known_hardlinks(torrents_path)?;
        Logger::debug(
            Category::HandleOrphaned,
            format!("Found {} unique files ({} total) in torrent folder", known_hardlinks.len(), known_hardlinks.values().sum::<u64>()).as_str(),
        );

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
                if protect_external_hardlinks {
                    if let Some(path_str) = path.to_str() {
                        let has_external_hardlinks = FileUtils::has_external_hardlinks(&known_hardlinks, path_str).context("get_orphaned_path_strings: Failed to get external hardlinks")?;
                        if has_external_hardlinks {
                            Logger::debug(Category::HandleOrphaned, format!("Ignoring path (has external hardlinks) {}", path_str).as_str());
                        } else {
                            is_orphan = true;
                        }
                    } else {
                        anyhow::bail!("Failed to get string from path (may due to non-UTF8 path: {:?}", path);
                    }
                }
                // Check for external hardlinks
                else {
                    is_orphan = true;
                }
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
                anyhow::bail!("path is neither file or dir: {:?}", path);
            }

            if is_orphan {
                if let Some(path_str) = path.to_str() {
                    Logger::debug(Category::HandleOrphaned, format!("Path is orphaned: {}", path_str).as_str());
                    orphaned_path_strings.insert(path_str.to_string());
                } else {
                    anyhow::bail!("Failed to get string from path (may due to non-UTF8 path: {:?}", path);
                }
            }
        }
        Logger::info(Category::HandleOrphaned, format!("Received {} orphaned paths", orphaned_path_strings.len()).as_str());

        Ok(orphaned_path_strings)
    }

    /**
     * Get dirs & files (including content) of all torrents
     */
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
            let torrent_files = torrent_manager.get_torrent_files(torrent.hash()).await.context("Failed to get torrent files")?;
            for torrent_file in torrent_files {
                let path_str = format!("{}/{}", torrent.save_path(), torrent_file.name());
                let path_buf = Path::new(&path_str).to_path_buf();
                if let Some(p) = path_buf.parent() {
                    torrent_paths.insert(p.to_path_buf());
                }
                torrent_paths.insert(path_buf);
            }
        }
        Logger::debug(Category::HandleOrphaned, format!("Received {} unique torrent paths", torrent_paths.len()).as_str());

        Ok(torrent_paths)
    }
}
