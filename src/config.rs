use crate::PROJECT_NAME;
use anyhow::{anyhow, Error, Result};
use directories_next::ProjectDirs;
use keybear_core::crypto::StaticSecretExt;
use log::debug;
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};

/// Default static key file filename.
const DEFAULT_SECRET_KEY_FILENAME: &str = "keybear.sk";
/// Default server public key file filename.
const DEFAULT_SERVER_PUBLIC_KEY_FILENAME: &str = "server.pk";
/// Client ID file filename.
const DEFAULT_ID_FILENAME: &str = "keybear.id";

/// The application configuration file.
#[derive(Debug, Default, Eq, PartialEq, Deserialize)]
pub struct Config {
    /// Keybear client name.
    name: String,

    /// Keybear server URL.
    url: String,

    /// Where the secret key lives.
    #[serde(default = "default_secret_key_path")]
    secret_key_path: PathBuf,

    /// Where the public key of the server lives.
    #[serde(default = "default_server_public_key_path")]
    server_public_key_path: PathBuf,

    /// Where the ID to communicate with the server lives.
    #[serde(default = "default_id_path")]
    id_path: PathBuf,

    /// Tor SOCKS5 proxy port.
    #[serde(default = "default_proxy_port")]
    proxy_port: u16,
}

impl Config {
    /// Load and parse a TOML configuration file.
    pub fn from_file<P>(file: &P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        // Get the generic as the actual reference so it's traits can be used
        let file = file.as_ref();

        debug!("Loading configuration file {:?}", file);

        // Attempt to open the configuration file
        let contents = fs::read_to_string(file)
            .map_err(|err| anyhow!("reading configuration file {:?} failed: {}", file, err))?;

        Self::from_str(&contents)
    }

    /// The name of this client.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The onion URL of the Keybear server.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// The Tor SOCKS5 proxy port.
    pub fn proxy_port(&self) -> u16 {
        self.proxy_port
    }

    /// Create a new secret key and save it to the file.
    pub fn generate_secret_key(&self) -> Result<StaticSecret> {
        debug!(
            "Creating secret key to be saved at {:?}",
            &self.secret_key_path
        );

        // Generate the key
        let secret_key = StaticSecret::new_with_os_rand();

        // Create the folder it belongs to
        fs::create_dir_all(self.secret_key_path.parent().ok_or_else(|| {
            anyhow!(
                "invalid secret key path {:?} has no parent directory",
                self.secret_key_path
            )
        })?)?;

        // Save it
        secret_key.save(&self.secret_key_path)?;

        Ok(secret_key)
    }

    /// Load the secret key and the server public key and create a shared key.
    pub fn shared_key(&self) -> Result<SharedSecret> {
        Ok(self
            .secret_key()?
            .diffie_hellman(&self.server_public_key()?))
    }

    /// Load the secret key from it's file.
    pub fn secret_key(&self) -> Result<StaticSecret> {
        StaticSecret::from_file(&self.secret_key_path)
    }

    /// Create a new secret key and save it to the file.
    pub fn save_id(&self, client_id: &str) -> Result<()> {
        debug!("Saving client ID at {:?}", &self.id_path);

        // Create the folder it belongs to
        fs::create_dir_all(self.id_path.parent().ok_or_else(|| {
            anyhow!("invalid ID path {:?} has no parent directory", self.id_path)
        })?)?;

        // Try to write the keys as raw bytes to the disk
        fs::write(&self.id_path, &client_id)
            .map_err(|err| anyhow!("Could not write ID to file {:?}: {}", self.id_path, err))?;

        Ok(())
    }

    /// Save the public key of the server.
    pub fn save_server_public_key(&self, server_public_key: &PublicKey) -> Result<()> {
        debug!(
            "Saving server public key at {:?}",
            &self.server_public_key_path
        );

        // Create the folder it belongs to
        fs::create_dir_all(self.server_public_key_path.parent().ok_or_else(|| {
            anyhow!(
                "invalid public server key path {:?} has no parent directory",
                self.server_public_key_path
            )
        })?)?;

        // Try to write the key as raw bytes to the disk
        fs::write(&self.server_public_key_path, server_public_key.to_bytes()).map_err(|err| {
            anyhow!(
                "Could not write public server key to file {:?}: {}",
                self.server_public_key_path,
                err
            )
        })?;

        Ok(())
    }

    /// Load the secret key from it's file.
    pub fn id(&self) -> Result<String> {
        fs::read_to_string(&self.id_path).map_err(|err| {
            anyhow!(
                "could not read client ID from file {:?}: {}",
                self.id_path,
                err
            )
        })
    }

    /// Check if the secret key already exists.
    pub fn id_exists(&self) -> bool {
        self.id_path.exists()
    }

    /// Load the server public key from file.
    fn server_public_key(&self) -> Result<PublicKey> {
        debug!(
            "Loading server public key from file {:?}",
            self.server_public_key_path
        );

        // Read the file
        let mut f = File::open(&self.server_public_key_path).map_err(|err| {
            anyhow!(
                "Reading crypto keys from file {:?} failed: {}",
                self.server_public_key_path,
                err
            )
        })?;

        // Read exactly the bytes from the file
        let mut bytes = [0; 32];
        f.read_exact(&mut bytes).map_err(|err| {
            anyhow!(
                "Server public key file {:?} has wrong size, it might be corrupt: {}",
                self.server_public_key_path,
                err
            )
        })?;

        // Try to construct the public key from the bytes
        Ok(PublicKey::from(bytes))
    }
}

impl FromStr for Config {
    type Err = Error;

    fn from_str(toml: &str) -> Result<Self> {
        toml::from_str(toml).map_err(|err| anyhow!("Reading keybear configuration failed: {}", err))
    }
}

/// The default Tor SOCKS5 proxy port.
fn default_proxy_port() -> u16 {
    9050
}

/// The default path where the secret key lives.
fn default_secret_key_path() -> PathBuf {
    ProjectDirs::from(PROJECT_NAME.0, PROJECT_NAME.1, PROJECT_NAME.2)
        .expect("No valid home directory found")
        .data_dir()
        .join(DEFAULT_SECRET_KEY_FILENAME)
}

/// The default path where the public key of the server lives.
fn default_server_public_key_path() -> PathBuf {
    ProjectDirs::from(PROJECT_NAME.0, PROJECT_NAME.1, PROJECT_NAME.2)
        .expect("No valid home directory found")
        .data_dir()
        .join(DEFAULT_SERVER_PUBLIC_KEY_FILENAME)
}

/// The default path where the ID to communicate with the server lives.
fn default_id_path() -> PathBuf {
    ProjectDirs::from(PROJECT_NAME.0, PROJECT_NAME.1, PROJECT_NAME.2)
        .expect("No valid home directory found")
        .data_dir()
        .join(DEFAULT_ID_FILENAME)
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use anyhow::Result;
    use std::str::FromStr;

    #[test]
    fn from_toml() -> Result<()> {
        let config = Config::from_str(
            r#"
            name = "client 1"
            url = "test.onion"
            proxy_port = 1234
        "#,
        )?;
        assert_eq!(config.name(), "client 1");
        assert_eq!(config.url(), "test.onion");
        assert_eq!(config.proxy_port(), 1234);

        // Verify that we get errors when an invalid config is used
        assert!(Config::from_str("*invalid*").is_err());

        Ok(())
    }
}
