use std::{collections::HashMap, os::unix::fs::MetadataExt, path::Path};

use anyhow::Context;
use walkdir::WalkDir;

pub struct FileUtils {}

impl FileUtils {
    /**
     * Returns a HashMap of the inode and the count of known links
     */
    pub fn get_known_hardlinks(dir_path_str: &str) -> Result<HashMap<u64, u64>, anyhow::Error> {
        let mut files_hardlink_count: HashMap<u64, u64> = HashMap::new();
        for entry in WalkDir::new(dir_path_str) {
            let entry_result = entry.context("Failed to get entry_result")?;
            if entry_result.file_type().is_file() {
                let entry_path = entry_result.into_path();
                let metadata = entry_path.metadata().context("Failed to get file metadata")?;
                *files_hardlink_count.entry(metadata.ino()).or_insert(0) += 1;
            }
        }
        Ok(files_hardlink_count)
    }

    pub fn has_external_hardlinks(known_hardlinks: &HashMap<u64, u64>, path_str: &str) -> Result<bool, anyhow::Error> {
        let path = Path::new(path_str);

        if !path.exists() {
            return Err(anyhow::anyhow!("path does not exist: {}", path_str));
        }

        // Handle file path
        if path.is_file() {
            let metadata = path.metadata().context("Failed to get file metadata")?;
            let ino = metadata.ino();
            let nlink = metadata.nlink();
            if let Some(known_links_count) = known_hardlinks.get(&ino) {
                if *known_links_count > nlink {
                    return Err(anyhow::anyhow!("{} | known_hardlinks_count ({}) is bigger than nlink ({}) which is impossible", path_str, known_links_count, nlink));
                }
                return Ok(*known_links_count != nlink);
            } else {
                return Err(anyhow::anyhow!("Didn't find file in known_hardlinks for {}", path_str));
            }
        }
        // Handle dir dir_path_str
        else if path.is_dir() {
            for entry in WalkDir::new(path_str) {
                let entry_result = entry.context("Failed to get entry result")?;
                if entry_result.file_type().is_file() {
                    let metadata = entry_result.metadata().context("Failed to get file metadata")?;
                    let ino = metadata.ino();
                    let nlink = metadata.nlink();
                    if let Some(known_links_count) = known_hardlinks.get(&ino) {
                        if *known_links_count > nlink {
                            return Err(anyhow::anyhow!("{} | known_hardlinks_count ({}) is bigger than nlink ({}) which is impossible", path_str, known_links_count, nlink));
                        }
                        if *known_links_count != nlink {
                            return Ok(true);
                        }
                    } else {
                        return Err(anyhow::anyhow!("Didn't find file in known_hardlinks for {}", path_str));
                    }
                }
            }
        }
        // Handle edge case not file or dir (should not happen)
        else {
            return Err(anyhow::anyhow!("path is neither file or dir: {}", path_str));
        }

        Ok(false)
    }
}
