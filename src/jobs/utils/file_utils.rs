use std::{os::unix::fs::MetadataExt, path::Path};

use anyhow::Context;
use walkdir::WalkDir;

use crate::logger::logger::Logger;

pub struct FileUtils {}

impl FileUtils {
    pub fn get_media_file_inodes(media_folder_path: &str) -> Vec<u64> {
        let mut media_inodes: Vec<u64> = Vec::new();
        for entry in WalkDir::new(media_folder_path) {
            let entry_result = match entry {
                Ok(entry_result) => entry_result,
                Err(e) => {
                    Logger::error(format!("Failed to get entry_result: {}", e).as_str());
                    continue;
                }
            };
            let path = entry_result.path();
            if path.is_file() {
                let metadata = match entry_result.metadata() {
                    Ok(metadata) => metadata,
                    Err(e) => {
                        Logger::error(format!("Failed to get file metadata: {}", e).as_str());
                        continue;
                    }
                };
                let inode = metadata.ino();
                media_inodes.push(inode);
            }
        }
        media_inodes
    }

    pub fn is_torrent_in_media_library(data_folder_path: &str, torrent_content_path: &str, media_file_inodes: &Vec<u64>) -> Result<bool, anyhow::Error> {
        let content_path_str = format!("{}{}", data_folder_path, torrent_content_path);
        let content_path = Path::new(content_path_str.as_str());

        if !content_path.exists() {
            return Err(anyhow::anyhow!("content_path does not exist: {}", content_path_str));
        }

        // Handle file content_path
        if content_path.is_file() {
            let metadata = content_path.metadata().context(format!("Failed to get metadata of content_path file: {}", content_path_str))?;
            let inode = metadata.ino();
            return Ok(media_file_inodes.contains(&inode));
        }
        // Handle dir content_path
        else if content_path.is_dir() {
            for entry in WalkDir::new(content_path_str.clone()) {
                let entry_result = entry.context("Failed to get entry result")?;
                let path = entry_result.path();
                if path.is_file() {
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
            return Err(anyhow::anyhow!("content_path is neither file or dir: {}", content_path_str));
        }

        Ok(false)
    }
}
