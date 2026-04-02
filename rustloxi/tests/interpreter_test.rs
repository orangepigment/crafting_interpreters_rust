use std::{env, path::PathBuf, process::Command};

#[test]
fn test_implementation() {
    // TODO: handle as PathBuf for type-safety and validation
    let repo_root = env::current_dir()
        .map(|cd| {
            cd.parent()
                .map(PathBuf::from)
                .and_then(|rr| rr.to_str().map(String::from))
        })
        .expect("Failed to get current directory")
        .expect("Failed to get repo root directory");

    //let test_tools_location = "<path>/craftinginterpreters";
    let test_tools_location =
        env::var("TEST_TOOLS_LOCATION").expect("TEST_TOOLS_LOCATION env variable is not defined");

    //let suite_name = "chap04_scanning";
    let suite_name = env::var("SUITE_NAME").expect("SUITE_NAME env variable is not defined");

    let test_status = Command::new("dart")
        .current_dir(&test_tools_location)
        .arg(format!("{test_tools_location}/tool/bin/test.dart"))
        .arg(suite_name)
        .arg("--interpreter")
        .arg(format!("{repo_root}/target/debug/rustloxi"))
        .status()
        .expect("Failed to execute command");

    assert!(test_status.success())
}
