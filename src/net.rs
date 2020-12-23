use crate::config::Config;
use anyhow::{anyhow, ensure, Result};
use keybear_core::crypto;
use log::debug;
use reqwest::{Client as HttpClient, Proxy, Url};
use serde::{de::DeserializeOwned, Serialize};

/// HTTP client with a SOCKS5 proxy to connect with the Keybear server over Tor.
pub struct Client {
    /// The reqwest client to make the HTTP connections.
    client: HttpClient,
    /// The server URL extracted from the config.
    url: String,
}

impl Client {
    /// Setup a client with the information from the configuration file.
    pub fn new(config: &Config) -> Result<Self> {
        debug!("Setting up HTTP client to connect to Tor proxy");

        // Configure the SOCKS5 Url with the custom port
        let mut url = Url::parse("socks5h://127.0.0.1")?;
        url.set_port(Some(config.proxy_port()))
            .map_err(|_| anyhow!("Could not set port {} on URL", config.proxy_port()))?;

        // Setup the Tor SOCKS5 proxy
        let proxy = Proxy::all(url)?;

        // Setup the HTTP client that uses the Tor proxy
        let client = HttpClient::builder().proxy(proxy).build()?;

        Ok(Self {
            client,
            url: config.url().to_string(),
        })
    }

    /// Make an un-encrypted POST request and get a response back.
    pub async fn unencrypted_post<S, D>(&self, path: &str, request_object: &S) -> Result<D>
    where
        S: Serialize,
        D: DeserializeOwned,
    {
        // Build the URL with the path
        let url = Url::parse(&format!("http://{}:5219", &self.url))?.join(path)?;

        let response = self
            .client
            // Create a POST request
            .post(url)
            // Add the object as a JSON payload
            .json(request_object)
            // Send it
            .send()
            .await?;

        // Throw the server error when the status code isn't in the 200-299 range
        ensure!(
            response.status().is_success(),
            "{}: {}",
            response.status().to_string(),
            response.text().await?
        );

        response
            // Try to convert the response to JSON
            .json()
            .await
            .map_err(|err| anyhow!(err))
    }
}
