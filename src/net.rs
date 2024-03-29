use crate::config::Config;
use anyhow::{anyhow, ensure, Result};
use keybear_core::{
    crypto::{self, Nonce},
    route::v1,
    CLIENT_ID_HEADER,
};
use log::{debug, trace};
use reqwest::{Client as HttpClient, Method, Proxy, Url};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

// Request timeout duration in seconds.
const REQUEST_TIMEOUT: u64 = 10;

/// Add a constructor to the reqwest client that sets up a proxied connection.
pub trait ProxyClient {
    /// Setup a HTTP client with a proxy connection from the settings defined in the configuration
    /// file.
    fn new_proxy(config: &Config) -> Result<Self>
    where
        Self: Sized;
}

/// HTTP client with a SOCKS5 proxy to connect with the Keybear server over Tor.
pub struct Client<'a> {
    /// The reqwest client to make the HTTP connections.
    client: HttpClient,
    /// The configuration file.
    ///
    /// It's a reference with a lifetime so the lifetime of this is bound to that of the
    /// configuration.
    config: &'a Config,
}

impl<'a> Client<'a> {
    /// Setup a client with the information from the configuration file.
    pub fn new(config: &'a Config) -> Result<Self> {
        Ok(Self {
            config,
            client: HttpClient::new_proxy(config)?,
        })
    }

    /// Send an encrypted POST request.
    pub async fn post<S, P, D>(&self, path: S, payload: &P) -> Result<D>
    where
        P: Serialize,
        D: DeserializeOwned,
        S: AsRef<str>,
    {
        self.request(path.as_ref(), Some(payload), Method::POST)
            .await
    }

    /// Send an encrypted GET request.
    pub async fn get<P, D, S>(&self, path: S, payload: Option<&P>) -> Result<D>
    where
        P: Serialize,
        D: DeserializeOwned,
        S: AsRef<str>,
    {
        self.request(path.as_ref(), payload, Method::GET).await
    }

    /// Send an encrypted DELETE request.
    pub async fn delete<S, P, D>(&self, path: S, payload: &P) -> Result<D>
    where
        P: Serialize,
        D: DeserializeOwned,
        S: AsRef<str>,
    {
        self.request(path.as_ref(), Some(payload), Method::DELETE)
            .await
    }

    /// Perform a request with an unspecified method.
    async fn request<P, D>(&self, path: &str, payload: Option<&P>, method: Method) -> Result<D>
    where
        P: Serialize,
        D: DeserializeOwned,
    {
        debug!("Trying to get nonce to make a request to \"{}\"", path);

        // Build the proxy URL for the nonce
        let url = proxy_url(&self.config.url(), v1::NONCE)?;

        // Build the request for the nonce
        let request = self
            .client
            .request(Method::GET, url)
            .timeout(Duration::new(REQUEST_TIMEOUT, 0))
            .header(CLIENT_ID_HEADER, self.config.id()?);

        // Send it
        let response = request.send().await?;

        trace!("Response received for nonce request");

        // Throw the server error when the status code isn't in the 200-299 range
        ensure!(
            response.status().is_success(),
            "{}: {}",
            response.status().to_string(),
            response.text().await?
        );

        // Get the bytes from the response
        let bytes = response.json::<[u8; 12]>().await?;

        // Throw an error when we got something else back then 12 bytes (the size of the nonce)
        ensure!(bytes.len() == 12, "Nonce response size is incorrect");

        // Construct the nonce
        let nonce = Nonce::from_slice(&bytes);

        debug!("Creating {} request to \"{}\"", &method, path);

        // Get the shared key to encrypt and decrypt
        let shared_key = self.config.shared_key()?;

        // Build the proxy URL
        let url = proxy_url(&self.config.url(), path)?;

        // Build the request
        let request = self
            .client
            .request(method, url)
            .timeout(Duration::new(REQUEST_TIMEOUT, 0))
            .header(CLIENT_ID_HEADER, self.config.id()?);

        // Add the object as an encrypted payload if applicable
        let request = if let Some(payload) = payload {
            trace!("Encrypting payload");

            // Try to encrypt the payload
            let encrypted = crypto::encrypt(&shared_key, &nonce, payload)?;

            request.body(encrypted)
        } else {
            request
        };

        trace!("Sending request");

        // Send it
        let response = request.send().await?;

        trace!("Response received");

        // Throw the server error when the status code isn't in the 200-299 range
        ensure!(
            response.status().is_success(),
            "{}: {}",
            response.status().to_string(),
            response.text().await?
        );

        // Get the bytes from the response
        let bytes = response.bytes().await?;

        // Try to decrypt the response
        crypto::decrypt(&shared_key, &nonce, &bytes)
    }
}

impl ProxyClient for HttpClient {
    fn new_proxy(config: &Config) -> Result<Self> {
        trace!("Setting up HTTP client to connect to Tor proxy");

        // Configure the SOCKS5 Url with the custom port
        let mut url = Url::parse("socks5h://127.0.0.1")?;
        url.set_port(Some(config.proxy_port()))
            .map_err(|_| anyhow!("could not set port {} on URL", config.proxy_port()))?;

        // Setup the Tor SOCKS5 proxy
        let proxy = Proxy::all(url)?;

        // Setup the HTTP client that uses the Tor proxy
        HttpClient::builder()
            .proxy(proxy)
            .build()
            .map_err(|err| anyhow!("could not build proxy http client: {}", err))
    }
}

/// Build a proxy URL from a path.
pub fn proxy_url(url: &str, path: &str) -> Result<Url> {
    Url::parse(&format!("http://{}:5219", url))?
        .join(path)
        .map_err(|err| anyhow!("could not build proxy URL: {}", err))
}
