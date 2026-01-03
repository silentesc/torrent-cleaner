use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct TorrentFile {
    name: String,
}

impl TorrentFile {
    pub fn name(&self) -> &str {
        &self.name
    }
}
