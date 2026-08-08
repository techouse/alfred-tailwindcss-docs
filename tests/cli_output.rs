use std::process::Command;

#[test]
fn argument_error_renders_valid_alfred_json_and_exits_two() {
    let output = Command::new(env!("CARGO_BIN_EXE_alfred_tailwindcss_docs"))
        .arg("--unknown")
        .output()
        .expect("workflow binary must run");
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout must be valid JSON");

    assert_eq!(
        (output.status.code(), json["items"][0]["title"].as_str()),
        (Some(2), Some("unknown argument: --unknown"))
    );
}

#[test]
fn verbose_runtime_error_renders_valid_alfred_json_and_writes_stderr() {
    let output = Command::new(env!("CARGO_BIN_EXE_alfred_tailwindcss_docs"))
        .args(["--verbose", "--query", "tls-verifier-runtime-error"])
        .env("ALGOLIA_APPLICATION_ID", "")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("workflow binary must run");
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout must be valid JSON");
    let stderr = String::from_utf8(output.stderr).expect("stderr must be valid UTF-8");

    assert_eq!(
        (output.status.code(), json["items"][0]["title"].as_str()),
        (Some(1), Some("ALGOLIA_APPLICATION_ID must not be empty"))
    );
    assert!(stderr.contains("ALGOLIA_APPLICATION_ID must not be empty"));
}
