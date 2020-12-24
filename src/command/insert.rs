use crate::{config::Config, net::Client};
use anyhow::Result;
use keybear_core::types::{PasswordResponse, RegisterPasswordRequest};
use log::info;

/// Handle the invoked command.
pub async fn insert(config: Config, name: &str, password: &str, echo: bool) -> Result<()> {
    info!("Inserting new password");

    // Setup the HTTP client
    let client = Client::new(&config)?;

    // Build the request object
    let request = RegisterPasswordRequest::new::<_, _, String, String>(name, password, None, None);

    // Request the password
    let response: PasswordResponse = client.post("v1/passwords", &request).await?;

    info!(
        "Password successfully added with ID: {}",
        response.password()
    );

    // Echo the password if requested
    if echo {
        println!("{}", password);
    }

    Ok(())
}
