#![forbid(unsafe_code)]

mod command;

use anyhow::{anyhow, bail, Result};
use clap::clap_app;

fn main() -> Result<()> {
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

        (@subcommand init =>
            (about: "Configure a connection to the keybear server")
            (@setting DisableVersion)
            (@arg URL: +required "Sets the server .onion URL")
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

    // Use the proper subcommand module for the invoked subcommand.
    match matches
        .subcommand()
        .ok_or_else(|| anyhow!("No subcommand invoked"))?
    {
        // kb init
        ("init", subcommand) => {
            let url = subcommand.value_of_t_or_exit::<String>("URL");

            command::init(&url)
        }
        // kb show
        ("show", subcommand) => {
            let url = subcommand.value_of_t_or_exit::<String>("URL");

            command::show(&url)
        }
        // kb ls
        ("ls", _) => command::ls(),
        // kb find
        ("find", _) => command::find(),
        // kb generate
        ("generate", subcommand) => {
            let name = subcommand.value_of_t_or_exit::<String>("NAME");
            let length = subcommand.value_of_t_or_exit::<usize>("length");
            let echo = subcommand.is_present("echo");

            command::generate(&name, length, echo)
        }
        // kb insert
        ("insert", subcommand) => {
            let name = subcommand.value_of_t_or_exit::<String>("NAME");
            let password = subcommand.value_of_t_or_exit::<String>("PASSWORD");
            let length = subcommand.value_of_t_or_exit::<usize>("length");
            let echo = subcommand.is_present("echo");

            command::insert(&name, &password, length, echo)
        }
        // kb edit
        ("edit", subcommand) => {
            let name = subcommand.value_of_t_or_exit::<String>("NAME");

            command::edit(&name)
        }
        // kb rm
        ("rm", subcommand) => {
            let name = subcommand.value_of_t_or_exit::<String>("NAME");

            command::rm(&name)
        }
        (other, _) => bail!("Unrecognized subcommand \"{}\"", other),
    }?;

    Ok(())
}
