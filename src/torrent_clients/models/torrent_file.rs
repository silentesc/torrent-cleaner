use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct TorrentFile {
    name: String,
    size: u64,
}

impl TorrentFile {
    pub fn name(&self) -> &str {
        &self.name
    }

    /**
     * Size in bytes
     */
    pub fn size(&self) -> &u64 {
        &self.size
    }
}
