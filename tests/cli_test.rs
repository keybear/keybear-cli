use assert_cmd::Command;

#[test]
fn no_subcommand() {
    // The command without any subcommand should show the help
    Command::cargo_bin("kb").unwrap().assert().failure();
}

#[test]
fn init() {
    let mut cmd = Command::cargo_bin("kb").unwrap();

    cmd.args(&["init", "test.onion"]).assert().success();
}
