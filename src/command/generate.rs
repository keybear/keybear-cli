use crate::{config::Config, net::Client};
use anyhow::Result;
use chbs::{config::BasicConfig, scheme::ToScheme};
use keybear_core::{
    route::v1,
    types::{PublicPassword, RegisterPasswordRequest},
};
use log::info;

/// Handle the invoked command.
pub async fn generate(config: Config, name: &str, length: usize, echo: bool) -> Result<()> {
    info!("Generating and inserting new password");

    // Setup the HTTP client
    let client = Client::new(&config)?;

    // Generate the password
    let mut config = BasicConfig::default();
    config.words = length;
    let password = config.to_scheme().generate();

    // Build the request object
    let request = RegisterPasswordRequest::new::<_, _, String, String>(name, &password, None, None);

    // Request the password
    let response: PublicPassword = client.post(v1::PASSWORD, &request).await?;

    info!("Password successfully added with ID: {}", response.id());

    // Echo the password if requested
    if echo {
        println!("{}", password);
    }

    Ok(())
}
