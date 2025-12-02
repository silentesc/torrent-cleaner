use std::sync::Arc;
use std::time::Duration;

use crate::logger::logger::Logger;
use crate::torrent_clients::models::torrent::{Torrent, TorrentInfo};
use crate::torrent_clients::models::tracker::Tracker;
use crate::torrent_clients::traits::torrent_client::TorrentClient;

use anyhow::Context;
use reqwest::{Client, RequestBuilder, Response, StatusCode, Url};
use tokio::time::sleep;

#[derive(Clone)]
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
                            Logger::warn(format!("Request to qbittorrent returned status code {}, trying to relogin", respone.status(),).as_str());
                            match self.login().await {
                                Ok(_) => {}
                                Err(e) => return Err(e),
                            }
                            sleep(delay).await;
                            continue;
                        }
                        // Any other non-successful status code
                        else {
                            Logger::warn(
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
                    Logger::warn(
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
}

impl TorrentClient for Qbittorrent {
    /**
     * Login
     */
    async fn login(&self) -> Result<(), anyhow::Error> {
        let endpoint = self.base_url.join("api/v2/auth/login")?;
        let params = [("username", &self.username), ("password", &self.password)];
        let max_retries = 6;
        let delay = Duration::from_secs(60);

        for attempt in 1..=max_retries {
            match self.client.post(endpoint.clone()).form(&(params.clone())).send().await {
                Ok(response) => match response.headers().get("set-cookie") {
                    Some(_) => {
                        Logger::info("Logged into qbittorrent");
                        return Ok(());
                    }
                    None => return Err(anyhow::anyhow!("Failed to authenticate to qbittorrent")),
                },
                Err(_) if attempt < max_retries => {
                    Logger::warn(
                        format!(
                            "Failed to login to qbittorrent on try {}/{}, waiting for {} seconds",
                            attempt,
                            max_retries,
                            delay.as_secs(),
                        )
                        .as_str(),
                    );
                    sleep(delay).await;
                    continue;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to login to qbittorrent on try {}/{}: {:?}",
                        attempt,
                        max_retries,
                        e
                    ));
                }
            }
        }
        Err(anyhow::anyhow!("Login request to qbittorrent failed"))
    }

    /**
     * Logout
     */
    async fn logout(&self) -> Result<(), anyhow::Error> {
        let endpoint = self.base_url.join("api/v2/auth/logout")?;

        let make_request_builder = || self.client.post(endpoint.clone());

        self.make_request(make_request_builder).await.context("Qbittorrent logout failed")?;

        Ok(())
    }

    async fn get_all_torrents(self: &Arc<Self>) -> Result<Vec<Torrent<Qbittorrent>>, anyhow::Error> {
        let endpoint = self.base_url.join("api/v2/torrents/info")?;

        let make_request_builder = || self.client.get(endpoint.clone());

        let response = self.make_request(make_request_builder).await.context("Qbittorrent get torrents failed")?;
        let torrent_infos: Vec<TorrentInfo> = response.json().await.context("Parsing torrents failed")?;
        let torrents: Vec<Torrent<Qbittorrent>> = torrent_infos.into_iter().map(|info| Torrent::new(info, self.clone())).collect();

        Ok(torrents)
    }

    /**
     * Get all trackers of a torrent
     */
    async fn get_torrent_trackers(&self, torrent_hash: &str) -> Result<Vec<Tracker>, anyhow::Error> {
        let endpoint = self.base_url.join("api/v2/torrents/trackers")?;
        let params = [("hash", torrent_hash)];

        let make_request_builder = || self.client.get(endpoint.clone()).query(&params);

        let response = self.make_request(make_request_builder).await.context("Qbittorrent get trackers failed")?;
        let trackers: Vec<Tracker> = response.json().await.context("Qbittorrent Parsing trackers failed")?;

        Ok(trackers)
    }

    /**
     * Stop torrent
     */
    async fn stop_torrent(&self, torrent_hash: &str) -> Result<(), anyhow::Error> {
        let endpoint = self.base_url.join("api/v2/torrents/stop")?;
        let params = [("hashes", torrent_hash)];

        let make_request_builder = || self.client.post(endpoint.clone()).form(&params);

        self.make_request(make_request_builder).await.context("Qbittorrent stop torrent failed")?;

        Ok(())
    }

    /**
     * Delete torrent
     */
    async fn delete_torrent(&self, torrent_hash: &str, delete_files: bool) -> Result<(), anyhow::Error> {
        let endpoint = self.base_url.join("api/v2/torrents/delete")?;
        let params = [("hashes", torrent_hash), ("deleteFiles", &delete_files.to_string())];

        let make_request_builder = || self.client.post(endpoint.clone()).form(&params);

        self.make_request(make_request_builder)
            .await
            .context("Qbittorrent delete torrent failed")?;

        Ok(())
    }
}
