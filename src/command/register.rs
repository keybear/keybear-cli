use crate::{
    config::Config,
    net::{self, ProxyClient},
};
use anyhow::{anyhow, ensure, Result};
use keybear_core::{
    route::v1,
    types::{RegisterDeviceRequest, RegisterDeviceResponse},
};
use log::{error, info};
use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use std::process;
use x25519_dalek::PublicKey;

/// Handle the invoked command.
pub async fn register(config: Config) -> Result<()> {
    // Exit when we are already registered
    if config.id_exists() {
        error!("Client is already registered");

        process::exit(1);
    }

    info!("Registering client to keybear server");

    // Generate a new secret key and save it
    let secret_key = config.generate_secret_key()?;
    // Generate a public keey from the secret key
    let public_key = PublicKey::from(&secret_key);

    // Build the request object
    let request = RegisterDeviceRequest::new(config.name(), &public_key);

    // Register the client
    let response: RegisterDeviceResponse =
        unencrypted_post(&config, &format!("v1{}", v1::REGISTER), &request).await?;

    info!("Device succesfully registered as \"{}\"", response.name());

    // Save the server public key from the response
    config.save_server_public_key(&response.server_public_key()?)?;

    // Save the ID from the response
    config.save_id(response.id())?;

    Ok(())
}

/// Make an un-encrypted POST request and get a response back.
pub async fn unencrypted_post<S, D>(config: &Config, path: &str, request_object: &S) -> Result<D>
where
    S: Serialize,
    D: DeserializeOwned,
{
    // Setup an HTTP client using a Tor proxy
    let client = Client::new_proxy(&config)?;

    let response = client
        // Create a POST request
        .post(net::proxy_url(config.url(), path)?)
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
