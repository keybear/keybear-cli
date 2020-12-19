use assert_cmd::Command;
use std::{fs::File, io::Write};
use tempdir::TempDir;

#[test]
fn no_subcommand() {
    // The command without any subcommand should show the help
    Command::cargo_bin("kb").unwrap().assert().failure();
}

#[test]
fn init() {
    // Create a fake configuration file
    let tmp_dir = TempDir::new("kb").unwrap();
    let file_path = tmp_dir.path().join("config.toml");
    {
        let mut tmp_file = File::create(file_path.clone()).unwrap();
        writeln!(tmp_file, r#"url = "test.onion""#).unwrap();
    }

    let mut cmd = Command::cargo_bin("kb").unwrap();

    cmd.args(&["init", "test.onion", "-c", file_path.to_str().unwrap()])
        .assert()
        .success();
}
