use crate::{config::Config, net::Client};
use anyhow::Result;
use keybear_core::{route::v1, types::PublicPassword};
use log::info;

/// Handle the invoked command.
pub async fn ls(config: Config) -> Result<()> {
    info!("Retrieving all password names");

    // Setup the HTTP client
    let client = Client::new(&config)?;

    // Request the password
    let response: Vec<PublicPassword> = client
        .get::<(), _, _>(&format!("v1{}", v1::PASSWORD), None)
        .await?;

    // Print the passwords
    response.into_iter().for_each(|pass| {
        println!("name:\t{}", pass.name());
        println!("id:\t{}", pass.id());
        if let Some(email) = pass.email() {
            println!("email:\t{}", email);
        }
        if let Some(website) = pass.website() {
            println!("website:\t{}", website);
        }
    });

    Ok(())
}
