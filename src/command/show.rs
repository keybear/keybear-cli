use crate::{config::Config, net::Client};
use anyhow::Result;
use keybear_core::{
    route::v1,
    types::{PasswordRequest, PasswordResponse},
};
use log::info;

/// Handle the invoked command.
pub async fn show(config: Config, name: &str) -> Result<()> {
    info!("Retrieving password for name \"{}\"", name);

    // Setup the HTTP client
    let client = Client::new(&config)?;

    // Build the request object
    let request = PasswordRequest::from_name(name);

    // Request the password
    let response: PasswordResponse = client
        .post(format!("{}/{}", v1::PASSWORD, name), &request)
        .await?;

    println!("{}", response.password());

    Ok(())
}
