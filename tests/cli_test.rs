use assert_cmd::Command;
use std::{fs::File, io::Write};

#[test]
fn no_subcommand() {
    // The command without any subcommand should show the help
    Command::cargo_bin("kb").unwrap().assert().failure();
}

#[test]
fn register() {
    // Create a fake configuration file
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("config.toml");
    {
        let mut tmp_file = File::create(file_path.clone()).unwrap();
        writeln!(tmp_file, r#"url = "test.onion""#).unwrap();
    }

    // Register to a non-existing onion address
    let mut cmd = Command::cargo_bin("kb").unwrap();

    cmd.args(&["register", "-c", file_path.to_str().unwrap()])
        .assert()
        .success();
}
