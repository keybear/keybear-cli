use anyhow::{anyhow, Error, Result};
use log::info;
use serde::Deserialize;
use std::{fs, path::Path, str::FromStr};

/// The application configuration file.
#[derive(Debug, Default, Eq, PartialEq, Deserialize)]
pub struct Config {
    /// Keybear server URL.
    url: String,
}

impl Config {
    /// Load and parse a TOML configuration file.
    pub fn from_file<P>(file: &P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        // Get the generic as the actual reference so it's traits can be used
        let file = file.as_ref();

        info!("Loading configuration file {:?}", file);

        // Attempt to open the configuration file
        let contents = fs::read_to_string(file)
            .map_err(|err| anyhow!("Reading configuration file {:?} failed: {}", file, err))?;

        Self::from_str(&contents)
    }

    /// The onion URL of the Keybear server.
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl FromStr for Config {
    type Err = Error;

    fn from_str(toml: &str) -> Result<Self> {
        toml::from_str(toml).map_err(|err| anyhow!("Reading keybear configuration failed: {}", err))
    }
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
            url = "test.onion"
        "#,
        )?;
        assert_eq!(config.url(), "test.onion");

        // Verify that we get errors when an invalid config is used
        assert!(Config::from_str("*invalid*").is_err());

        Ok(())
    }
}
