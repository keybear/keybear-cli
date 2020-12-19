use crate::{config::Config, net};
use anyhow::{anyhow, ensure, Result};

/// Handle the invoked command.
pub async fn register(config: Config) -> Result<()> {
    // Setup an HTTP client using a Tor proxy
    let client = net::setup_client(&config)?;

    Ok(())
}
