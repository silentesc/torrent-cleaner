use std::time::Duration;

use crate::logger::{enums::category::Category, logger::Logger};
use crate::torrent_clients::models::torrent::Torrent;
use crate::torrent_clients::models::torrent_file::TorrentFile;
use crate::torrent_clients::models::tracker::Tracker;

use anyhow::Context;
use reqwest::{Client, RequestBuilder, Response, StatusCode, Url};
use tokio::time::sleep;

pub struct Qbittorrent {
    client: Client,
    base_url: Url,
    username: String,
    password: String,
}

impl Qbittorrent {
    /**
     * Create new qbittorrent client
     */
    pub fn new(base_url: &str, username: &str, password: &str) -> Result<Self, anyhow::Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(10))
            .cookie_store(true)
            .build()
            .context("Failed to build reqwest qbittorrent client")?;

        let url = Url::parse(base_url).context(format!("Invalid base url: {base_url}"))?;

        Ok(Self {
            client,
            base_url: url,
            username: String::from(username),
            password: String::from(password),
        })
    }

    /**
     * Make request with retry logic
     */
    async fn make_request<F>(&self, make_request_builder: F) -> Result<Response, anyhow::Error>
    where
        F: Fn() -> RequestBuilder,
    {
        let max_retries = 3;
        let delay = Duration::from_secs(3);

        for attempt in 0..=max_retries {
            match make_request_builder().send().await {
                // Request succeeded
                Ok(respone) => {
                    if respone.status().is_success() {
                        return Ok(respone);
                    } else if attempt < max_retries {
                        // Not logged in anymore (e.g. qbittorrent restarted)
                        if respone.status() == StatusCode::UNAUTHORIZED || respone.status() == StatusCode::FORBIDDEN {
                            Logger::error(Category::Qbittorrent, format!("Request to qbittorrent returned status code {}, trying to relogin", respone.status(),).as_str());
                            match self.login().await {
                                Ok(_) => {}
                                Err(e) => return Err(e),
                            }
                            continue;
                        }
                        // Any other non-successful status code
                        else {
                            Logger::error(
                                Category::Qbittorrent,
                                format!(
                                    "Request to qbittorrent returned status code {}, waiting for {} seconds to try again: {}",
                                    respone.status(),
                                    delay.as_secs(),
                                    respone.text().await.context("Failed to get error text")?,
                                )
                                .as_str(),
                            );
                            sleep(delay).await;
                            continue;
                        }
                    }
                }
                // Request failed
                Err(e) if attempt < max_retries => {
                    Logger::error(
                        Category::Qbittorrent,
                        format!(
                            "Request to qbittorrent failed on try {}/{}, waiting for {} seconds to try again: {}",
                            attempt + 1,
                            max_retries,
                            delay.as_secs(),
                            e.to_string(),
                        )
                        .as_str(),
                    );
                    sleep(delay).await;
                    continue;
                }
                Err(e) => return Err(anyhow::anyhow!(e)),
            }
        }
        Err(anyhow::anyhow!("Request to failed after {} tries", max_retries))
    }

    /**
     * Login
     */
    pub async fn login(&self) -> Result<(), anyhow::Error> {
        if self.is_logged_in().await? {
            Logger::warn(Category::Qbittorrent, "Login: Already logged in, ignoring...");
            return Ok(());
        }

        let endpoint = self.base_url.join("api/v2/auth/login")?;
        let params = [("username", &self.username), ("password", &self.password)];
        let max_retries = 6;
        let delay = Duration::from_secs(60);

        for attempt in 1..=max_retries {
            match self.client.post(endpoint.clone()).form(&(params.clone())).send().await {
                Ok(response) => match response.headers().get("set-cookie") {
                    Some(_) => {
                        Logger::info(Category::Qbittorrent, "Logged in");
                        return Ok(());
                    }
                    None => return Err(anyhow::anyhow!("Failed to authenticate to qbittorrent")),
                },
                Err(_) if attempt < max_retries => {
                    Logger::error(
                        Category::Qbittorrent,
                        format!("Failed to login to qbittorrent on try {}/{}, waiting for {} seconds", attempt, max_retries, delay.as_secs(),).as_str(),
                    );
                    sleep(delay).await;
                    continue;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to login to qbittorrent on try {}/{}: {:#}", attempt, max_retries, e));
                }
            }
        }
        Err(anyhow::anyhow!("Login request to qbittorrent failed"))
    }

    /**
     * Logout
     */
    pub async fn logout(&self) -> Result<(), anyhow::Error> {
        let endpoint = self.base_url.join("api/v2/auth/logout")?;

        let make_request_builder = || self.client.post(endpoint.clone());

        self.make_request(make_request_builder).await.context("Qbittorrent logout failed")?;

        Logger::info(Category::Qbittorrent, "Logged out");

        Ok(())
    }

    /**
     * Is logged in
     */
    pub async fn is_logged_in(&self) -> Result<bool, anyhow::Error> {
        let endpoint = self.base_url.join("api/v2/app/version")?;

        let response = self.client.get(endpoint.clone()).send().await.context("Qbittorrent getting app version failed")?;
        let text = response.text().await?;

        if text == "Forbidden" {
            return Ok(false);
        }

        Ok(true)
    }

    /**
     * Get all torrents
     */
    pub async fn get_all_torrents(&self) -> Result<Vec<Torrent>, anyhow::Error> {
        let endpoint = self.base_url.join("api/v2/torrents/info")?;

        let make_request_builder = || self.client.get(endpoint.clone());

        let response = self.make_request(make_request_builder).await.context("Qbittorrent get torrents failed")?;
        let torrents: Vec<Torrent> = response.json().await.context("Qbittorrent parsing torrents failed")?;

        Ok(torrents)
    }

    /**
     * Get all trackers of a torrent
     */
    pub async fn get_torrent_trackers(&self, torrent_hash: &str) -> Result<Vec<Tracker>, anyhow::Error> {
        let endpoint = self.base_url.join("api/v2/torrents/trackers")?;
        let params = [("hash", torrent_hash)];

        let make_request_builder = || self.client.get(endpoint.clone()).query(&params);

        let response = self.make_request(make_request_builder).await.context("Qbittorrent get trackers failed")?;
        let trackers: Vec<Tracker> = response.json().await.context("Qbittorrent parsing trackers failed")?;

        Ok(trackers)
    }

    /**
     * Get torrent files
     */
    pub async fn get_torrent_files(&self, torrent_hash: &str) -> Result<Vec<TorrentFile>, anyhow::Error> {
        let endpoint = self.base_url.join("api/v2/torrents/files")?;
        let params = [("hash", torrent_hash)];

        let make_request_builder = || self.client.get(endpoint.clone()).query(&params);

        let response = self.make_request(make_request_builder).await.context("Qbittorrent get files failed")?;
        let torrent_files: Vec<TorrentFile> = response.json().await.context("Qbittorrent parsing TorrentFile failed")?;

        Ok(torrent_files)
    }

    /**
     * Stop torrent
     */
    pub async fn stop_torrent(&self, torrent_hash: &str) -> Result<(), anyhow::Error> {
        let endpoint = self.base_url.join("api/v2/torrents/stop")?;
        let params = [("hashes", torrent_hash)];

        let make_request_builder = || self.client.post(endpoint.clone()).form(&params);

        self.make_request(make_request_builder).await.context("Qbittorrent stop torrent failed")?;

        Ok(())
    }

    /**
     * Delete torrent
     */
    pub async fn delete_torrent(&self, torrent_hash: &str, delete_files: bool) -> Result<(), anyhow::Error> {
        let endpoint = self.base_url.join("api/v2/torrents/delete")?;
        let params = [("hashes", torrent_hash), ("deleteFiles", &delete_files.to_string())];

        let make_request_builder = || self.client.post(endpoint.clone()).form(&params);

        self.make_request(make_request_builder).await.context("Qbittorrent delete torrent failed")?;

        Ok(())
    }
}
