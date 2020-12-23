use crate::{config::Config, net::Client};
use anyhow::{anyhow, ensure, Result};
use keybear_core::types::{RegisterDeviceRequest, RegisterDeviceResponse};
use log::{debug, error, info, warn};
use std::{io::Write, process};
use x25519_dalek::PublicKey;

/// Handle the invoked command.
pub async fn register(config: Config) -> Result<()> {
    // Exit when we are already registered
    if config.id_exists() {
        error!("Client is already registered");

        process::exit(1);
    }

    info!("Registering client to keybear server");

    // Setup an HTTP client using a Tor proxy
    let client = Client::new(&config)?;

    // Generate a new secret key and save it
    let secret_key = config.generate_secret_key()?;
    // Generate a public keey from the secret key
    let public_key = PublicKey::from(&secret_key);

    // Build the request object
    let request = RegisterDeviceRequest::new(config.name(), &public_key);

    // Register the client
    let response: RegisterDeviceResponse = client.unencrypted_post("v1/register", &request).await?;

    eprintln!("Device succesfully registered as \"{}\"", response.name());

    // Save the ID from the response
    config.save_id(response.id())?;

    Ok(())
}
