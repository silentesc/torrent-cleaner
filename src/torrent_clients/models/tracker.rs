use serde::Deserialize;

#[derive(Deserialize)]
pub struct Tracker {
    url: String,
    status: i8,
    msg: String,
}

impl Tracker {
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn status(&self) -> &i8 {
        &self.status
    }
    pub fn msg(&self) -> &str {
        &self.msg
    }
}
