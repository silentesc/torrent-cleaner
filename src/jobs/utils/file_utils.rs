use std::{collections::HashMap, os::unix::fs::MetadataExt, path::Path};

use anyhow::Context;
use walkdir::{DirEntryExt, WalkDir};

use crate::{logger::enums::category::Category, trace};

pub struct FileUtils {}

impl FileUtils {
    /**
     * Returns a HashMap of the inode and the count of known links
     * Walk through dir instead of using torrent content files because an orphaned file might still be externally linked
     */
    pub fn get_known_hardlinks(dir_path_str: &str) -> Result<HashMap<u64, u64>, anyhow::Error> {
        let mut files_hardlink_count: HashMap<u64, u64> = HashMap::new();
        for entry in WalkDir::new(dir_path_str) {
            let entry_result = entry.context("Failed to get entry_result")?;
            if entry_result.file_type().is_file() {
                *files_hardlink_count.entry(entry_result.ino()).or_insert(0) += 1;
            }
        }
        Ok(files_hardlink_count)
    }

    /**
     * Check if the given path has more hardlinks than the known amount of hardlinks in the HashMap
     * If the path is a file, check it directly, if the path is a dir walk through dir and all subdirs recursively and check each file
     */
    pub fn has_external_hardlinks(known_hardlinks: &HashMap<u64, u64>, path_str: &str) -> Result<bool, anyhow::Error> {
        let path_metadata = Path::new(path_str).metadata().context(format!("Failed to get file metadata for {}", path_str))?;
        let path_file_type = path_metadata.file_type();

        // Handle file path
        if path_file_type.is_file() {
            trace!(Category::FileUtils, "has_external_hardlinks: Path is file: {}", path_str);
            let ino = path_metadata.ino();
            let nlink = path_metadata.nlink();
            if let Some(known_links_count) = known_hardlinks.get(&ino) {
                trace!(Category::FileUtils, "  -> File path {} (ino {}) has known_links_count {} nlink {}", path_str, ino, known_links_count, nlink);
                if *known_links_count > nlink {
                    anyhow::bail!("{} | known_hardlinks_count ({}) is bigger than nlink ({}) which is impossible", path_str, known_links_count, nlink);
                }
                return Ok(*known_links_count != nlink);
            } else {
                anyhow::bail!("Didn't find file in known_hardlinks for {}", path_str);
            }
        }
        // Handle dir path
        else if path_file_type.is_dir() {
            trace!(Category::FileUtils, "has_external_hardlinks: Path is dir and will now be checked recursively: {}", path_str);
            for entry in WalkDir::new(path_str) {
                let entry_result = entry.context("Failed to get entry result")?;
                let entry_result_path = entry_result.path();
                let metadata = entry_result.metadata().context(format!("Failed to get file metadata for {:?}", entry_result_path))?;
                if metadata.is_file() {
                    let ino = metadata.ino();
                    let nlink = metadata.nlink();
                    match known_hardlinks.get(&ino) {
                        Some(known_links_count) => {
                            trace!(Category::FileUtils, "  -> File path {:?} (ino {}) has known_links_count {} nlink {}", entry_result_path, ino, known_links_count, nlink);
                            if *known_links_count > nlink {
                                anyhow::bail!("{} | known_hardlinks_count ({}) is bigger than nlink ({}) which is impossible", path_str, known_links_count, nlink);
                            }
                            // If known_links_count == nlink then all links are known. Ignore them here and return false if everything is checked and nothing is true
                            if *known_links_count < nlink {
                                return Ok(true);
                            }
                        }
                        None => anyhow::bail!("Didn't find file in known_hardlinks for {}", path_str),
                    }
                }
            }
        }
        // Handle edge case not file or dir (should not happen)
        else {
            anyhow::bail!("path is neither file or dir: {}", path_str);
        }

        Ok(false)
    }
}
