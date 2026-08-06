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
