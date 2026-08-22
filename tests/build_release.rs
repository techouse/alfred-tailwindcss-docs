#![cfg(unix)]

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use anyhow::{Context, Result, anyhow};
use tempfile::TempDir;

const CONFIGURATION_VARIABLES: [&str; 3] = [
    "ALGOLIA_APPLICATION_ID",
    "ALGOLIA_SEARCH_ONLY_API_KEY",
    "ALGOLIA_SEARCH_INDEX",
];

struct Fixture {
    directory: TempDir,
    record_path: PathBuf,
    path: std::ffi::OsString,
}

impl Fixture {
    fn new() -> Result<Self> {
        let directory = tempfile::tempdir()?;
        let root = directory.path();
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("Makefile"),
            root.join("Makefile"),
        )?;

        let fake_bin = root.join("fake-bin");
        fs::create_dir(&fake_bin)?;
        write_executable(
            &fake_bin.join("cargo"),
            "#!/usr/bin/env bash\nset -eu\nprintf '%s\\n' \"${ALGOLIA_APPLICATION_ID-}\" \"${ALGOLIA_SEARCH_ONLY_API_KEY-}\" \"${ALGOLIA_SEARCH_INDEX-}\" > \"$FAKE_CARGO_RECORD\"\n",
        )?;

        let scripts = root.join("scripts");
        fs::create_dir(&scripts)?;
        write_executable(
            &scripts.join("package-workflow.sh"),
            "#!/usr/bin/env bash\n",
        )?;

        let record_path = root.join("cargo-record");
        let inherited_path = env::var_os("PATH").unwrap_or_default();
        let path =
            env::join_paths(std::iter::once(fake_bin).chain(env::split_paths(&inherited_path)))?;

        Ok(Self {
            directory,
            record_path,
            path,
        })
    }

    fn command(&self) -> Command {
        let mut command = Command::new("make");
        command
            .arg("-C")
            .arg(self.directory.path())
            .arg("build-release")
            .env("PATH", &self.path)
            .env("FAKE_CARGO_RECORD", &self.record_path);
        for variable in CONFIGURATION_VARIABLES {
            command.env_remove(variable);
        }
        command
    }

    fn recorded_values(&self) -> Result<Vec<String>> {
        Ok(fs::read_to_string(&self.record_path)?
            .lines()
            .map(str::to_owned)
            .collect())
    }
}

fn write_executable(path: &Path, contents: &str) -> Result<()> {
    fs::write(path, contents)?;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

fn assert_success(output: &Output) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }

    Err(anyhow!(
        "make failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

#[test]
fn build_release_ignores_malformed_dotenv_when_runtime_is_complete() -> Result<()> {
    let fixture = Fixture::new()?;
    fs::write(
        fixture.directory.path().join(".env"),
        "BROKEN=\"unterminated\n",
    )?;

    let output = fixture
        .command()
        .env("ALGOLIA_APPLICATION_ID", "runtime-app")
        .env("ALGOLIA_SEARCH_ONLY_API_KEY", "runtime-key")
        .env("ALGOLIA_SEARCH_INDEX", "runtime-index")
        .output()?;
    assert_success(&output)?;

    assert_eq!(
        fixture.recorded_values()?,
        ["runtime-app", "runtime-key", "runtime-index"]
    );
    Ok(())
}

#[test]
fn build_release_preserves_runtime_values_and_fills_missing_values_from_dotenv() -> Result<()> {
    let fixture = Fixture::new()?;
    fs::write(
        fixture.directory.path().join(".env"),
        "ALGOLIA_APPLICATION_ID=dotenv-app\nALGOLIA_SEARCH_ONLY_API_KEY=dotenv-key\nALGOLIA_SEARCH_INDEX=dotenv-index\n",
    )?;

    let output = fixture
        .command()
        .env("ALGOLIA_APPLICATION_ID", "runtime-app")
        .output()?;
    assert_success(&output)?;

    assert_eq!(
        fixture.recorded_values()?,
        ["runtime-app", "dotenv-key", "dotenv-index"]
    );
    Ok(())
}

#[test]
fn build_release_rejects_empty_runtime_values_instead_of_using_dotenv() -> Result<()> {
    let fixture = Fixture::new()?;
    fs::write(
        fixture.directory.path().join(".env"),
        "ALGOLIA_APPLICATION_ID=dotenv-app\nALGOLIA_SEARCH_ONLY_API_KEY=dotenv-key\nALGOLIA_SEARCH_INDEX=dotenv-index\n",
    )?;

    let output = fixture
        .command()
        .env("ALGOLIA_APPLICATION_ID", "")
        .output()?;

    assert!(!output.status.success());
    assert!(!fixture.record_path.exists());
    Ok(())
}

#[test]
fn build_release_fails_on_malformed_dotenv_when_runtime_is_incomplete() -> Result<()> {
    let fixture = Fixture::new()?;
    fs::write(
        fixture.directory.path().join(".env"),
        "BROKEN=\"unterminated\n",
    )?;

    let output = fixture.command().output()?;

    assert!(!output.status.success());
    assert!(!fixture.record_path.exists());
    String::from_utf8(output.stderr)
        .context("make stderr must be valid UTF-8")
        .map(|_| ())
}

#[test]
fn build_release_fails_when_dotenv_fails_after_setting_values() -> Result<()> {
    let fixture = Fixture::new()?;
    fs::write(
        fixture.directory.path().join(".env"),
        "ALGOLIA_APPLICATION_ID=dotenv-app\nALGOLIA_SEARCH_ONLY_API_KEY=dotenv-key\nALGOLIA_SEARCH_INDEX=dotenv-index\nfalse\n",
    )?;

    let output = fixture.command().output()?;

    assert!(!output.status.success());
    assert!(!fixture.record_path.exists());
    Ok(())
}

#[test]
fn build_release_fails_when_dotenv_exits_early() -> Result<()> {
    let fixture = Fixture::new()?;
    fs::write(fixture.directory.path().join(".env"), "exit 0\n")?;

    let output = fixture.command().output()?;

    assert!(!output.status.success());
    assert!(!fixture.record_path.exists());
    let stderr = String::from_utf8(output.stderr).context("make stderr must be valid UTF-8")?;
    assert!(stderr.contains("must be set in the environment or .env file"));
    Ok(())
}

#[test]
fn build_release_prefers_local_dotenv_over_path_dotenv() -> Result<()> {
    let fixture = Fixture::new()?;
    fs::write(
        fixture.directory.path().join("fake-bin").join(".env"),
        "ALGOLIA_APPLICATION_ID=path-app\nALGOLIA_SEARCH_ONLY_API_KEY=path-key\nALGOLIA_SEARCH_INDEX=path-index\n",
    )?;
    fs::write(
        fixture.directory.path().join(".env"),
        "ALGOLIA_APPLICATION_ID=local-app\nALGOLIA_SEARCH_ONLY_API_KEY=local-key\nALGOLIA_SEARCH_INDEX=local-index\n",
    )?;

    let output = fixture.command().output()?;
    assert_success(&output)?;

    assert_eq!(
        fixture.recorded_values()?,
        ["local-app", "local-key", "local-index"]
    );
    Ok(())
}
