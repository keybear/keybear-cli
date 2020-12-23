#![forbid(unsafe_code)]

mod command;
mod config;
mod net;

use crate::config::Config;
use anyhow::{anyhow, bail, Result};
use clap::clap_app;
use directories_next::ProjectDirs;
use std::path::PathBuf;

/// Names used for the directory in the configuration folder.
pub const PROJECT_NAME: (&str, &str, &str) = ("com", "keybear", "keybear");

/// Environment variable name for the configuration file location.
pub const CONFIG_ENV_NAME: &str = "KEYBEAR_CONFIG";
/// Default configuration file filename.
pub const DEFAULT_CONFIG_FILENAME: &str = "keybear.toml";

/// Main application entry point.
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the panic handler
    human_panic::setup_panic!();

    let matches = clap_app!(keybear =>
        (version: clap::crate_version!())
        (author: clap::crate_authors!())
        (about: clap::crate_description!())

        // Show the help when a command is invoked without a subcommand
        (@setting SubcommandRequiredElseHelp)
        // Don't let subcommands have their own version
        (@setting GlobalVersion)
        // Throw an error when input isn't proper UTF-8
        (@setting StrictUtf8)
        // Panic when the help text is missing
        (@setting HelpRequired)
        // The order of the arguments is the order in which they are declared, not alphabetically
        (@setting DeriveDisplayOrder)
        // Enable colored output when using a terminal
        (@setting ColorAuto)
        // Colored help messages
        (@global_setting ColoredHelp)

        // The global configuration argument
        (@arg config: -c --config
            env(CONFIG_ENV_NAME)
            default_value(&default_config_path()?)
            global(true)
            "Path of the configuration file")
        // The global quiet argument
        (@arg quiet: -q --quiet
            global(true)
            "Silence all output")
        // The global verbosity argument
        (@arg verbosity: -v --verbose
            global(true)
            multiple_occurrences(true)
            "Verbose mode")

        (@subcommand register =>
            (about: "Register this client to the server")
            (@setting DisableVersion)
        )
        (@subcommand show =>
            (about: "Show an existing password")
            (@setting DisableVersion)
            (@arg NAME: +required "Name of the password")
        )
        (@subcommand ls =>
            (about: "List all passwords")
            (@setting DisableVersion)
        )
        (@subcommand find =>
            (about: "List passwords that match pass-names")
            (@setting DisableVersion)
        )
        (@subcommand generate =>
            (about: "Generate a new password")
            (@setting DisableVersion)
            (@arg NAME: +required "Name of the password")
            (@arg length: -l --length default_value("25") "Amount of characters to generate")
            (@arg echo: -e --echo "Echo the password back to the console")
        )
        (@subcommand insert =>
            (about: "Insert a new password")
            (@setting DisableVersion)
            (@arg NAME: +required "Name of the password")
            (@arg PASSWORD: +required "Actual password")
            (@arg echo: -e --echo "Echo the password back to the console")
        )
        (@subcommand edit =>
            (about: "Edit an existing password using a text editor")
            (@setting DisableVersion)
            (@arg NAME: +required "Name of the password")
        )
        (@subcommand rm =>
            (about: "Remove an existing password")
            (@setting DisableVersion)
            (@arg NAME: +required "Name of the password")
        )
    )
    .get_matches();

    // Initialize the logger
    stderrlog::new()
        .module(module_path!())
        .verbosity(matches.occurrences_of("verbosity") as usize)
        .quiet(matches.is_present("quiet"))
        .init()?;

    // Get the configuration argument
    let config_path: PathBuf = matches.value_of_t_or_exit("config");
    // Load the configuration file
    let config = Config::from_file(&config_path)?;

    // Use the proper subcommand module for the invoked subcommand.
    match matches
        .subcommand()
        .ok_or_else(|| anyhow!("No subcommand invoked"))?
    {
        // kb register
        ("register", _) => command::register(config).await,
        // kb show
        ("show", subcommand) => {
            let url = subcommand.value_of_t_or_exit::<String>("URL");

            command::show(config, &url)
        }
        // kb ls
        ("ls", _) => command::ls(config),
        // kb find
        ("find", _) => command::find(config),
        // kb generate
        ("generate", subcommand) => {
            let name = subcommand.value_of_t_or_exit::<String>("NAME");
            let length = subcommand.value_of_t_or_exit::<usize>("length");
            let echo = subcommand.is_present("echo");

            command::generate(config, &name, length, echo)
        }
        // kb insert
        ("insert", subcommand) => {
            let name = subcommand.value_of_t_or_exit::<String>("NAME");
            let password = subcommand.value_of_t_or_exit::<String>("PASSWORD");
            let length = subcommand.value_of_t_or_exit::<usize>("length");
            let echo = subcommand.is_present("echo");

            command::insert(config, &name, &password, length, echo)
        }
        // kb edit
        ("edit", subcommand) => {
            let name = subcommand.value_of_t_or_exit::<String>("NAME");

            command::edit(config, &name)
        }
        // kb rm
        ("rm", subcommand) => {
            let name = subcommand.value_of_t_or_exit::<String>("NAME");

            command::rm(config, &name)
        }
        (other, _) => bail!("Unrecognized subcommand \"{}\"", other),
    }?;

    Ok(())
}

/// Get the default configuration file location.
fn default_config_path() -> Result<String> {
    Ok(
        ProjectDirs::from(PROJECT_NAME.0, PROJECT_NAME.1, PROJECT_NAME.2)
            .ok_or_else(|| anyhow!("No valid home directory found"))?
            .config_dir()
            .join(DEFAULT_CONFIG_FILENAME)
            .to_str()
            .ok_or_else(|| anyhow!("Could not convert path to string"))?
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_config_directory() {
        // We should be able to get the default configuration directory on all OSes
        super::default_config_path().unwrap();
    }
}
