use crate::config::Config;
use anyhow::{anyhow, Result};
use reqwest::{Client, Proxy, Url};

/// Setup a HTTP client with a SOCKS5 proxy to connect with the Keybear server over Tor.
pub fn setup_client(config: &Config) -> Result<Client> {
    // Configure the SOCKS5 Url with the custom port
    let mut url = Url::parse("socks5h://127.0.0.1")?;
    url.set_port(Some(config.proxy_port()))
        .map_err(|_| anyhow!("Could not set port {} on URL", config.proxy_port()))?;

    // Setup the Tor SOCKS5 proxy
    let proxy = Proxy::all(url)?;

    // Setup the client that uses the Tor proxy
    Ok(Client::builder().proxy(proxy).build()?)
}
