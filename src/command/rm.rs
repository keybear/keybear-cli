use crate::{config::Config, net::Client};
use anyhow::Result;
use keybear_core::{
    route::v1,
    types::{PasswordRequest, PublicPassword},
};
use log::{debug, info};
use std::process;

/// Handle the invoked command.
pub async fn rm(config: Config, name: &str) -> Result<()> {
    info!("Removing password with name \"{}\"", name);

    // Setup the HTTP client
    let client = Client::new(&config)?;

    debug!("Requesting all passwords");
    // Request the password
    let passwords: Vec<PublicPassword> = client.get::<(), _, _>(v1::PASSWORD, None).await?;

    // Find the password matching the name
    let id = match passwords.iter().find(|password| password.name() == name) {
        Some(password) => password.id(),
        None => {
            eprintln!("Password with name \"{}\" does not exist", name);
            process::exit(1);
        }
    };

    // Build the request object
    let request = PasswordRequest::from_id(id);

    // Remove the password
    let _: () = client.delete(v1::PASSWORD, &request).await?;

    println!("Successfully removed password \"{}\"", name);

    Ok(())
}
