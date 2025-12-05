use std::{collections::HashSet, os::unix::fs::MetadataExt, path::Path};

use anyhow::Context;
use walkdir::WalkDir;

pub struct FileUtils {}

impl FileUtils {
    pub fn get_media_file_inodes(media_folder_path: &str) -> Result<HashSet<u64>, anyhow::Error> {
        let mut media_inodes: HashSet<u64> = HashSet::new();
        for entry in WalkDir::new(media_folder_path) {
            let entry_result = entry.context("Failed to get entry_result")?;
            if entry_result.file_type().is_file() {
                let metadata = entry_result.metadata().context("Failed to get file metadata")?;
                let inode = metadata.ino();
                media_inodes.insert(inode);
            }
        }
        Ok(media_inodes)
    }

    pub fn is_torrent_in_media_library(torrent_content_path: &str, media_file_inodes: &HashSet<u64>) -> Result<bool, anyhow::Error> {
        let content_path = Path::new(torrent_content_path);

        if !content_path.exists() {
            return Err(anyhow::anyhow!("content_path does not exist: {}", torrent_content_path));
        }

        // Handle file content_path
        if content_path.is_file() {
            let metadata = content_path.metadata().context(format!("Failed to get metadata of content_path file: {}", torrent_content_path))?;
            let inode = metadata.ino();
            return Ok(media_file_inodes.contains(&inode));
        }
        // Handle dir content_path
        else if content_path.is_dir() {
            for entry in WalkDir::new(torrent_content_path) {
                let entry_result = entry.context("Failed to get entry result")?;
                if entry_result.file_type().is_file() {
                    let metadata = entry_result.metadata().context("Failed to get entry result metadata")?;
                    let inode = metadata.ino();
                    if media_file_inodes.contains(&inode) {
                        return Ok(true);
                    }
                }
            }
        }
        // Handle edge case not file or dir (should not happen)
        else {
            return Err(anyhow::anyhow!("content_path is neither file or dir: {}", torrent_content_path));
        }

        Ok(false)
    }
}
