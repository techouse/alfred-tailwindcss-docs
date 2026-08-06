#[cfg(unix)]
use std::ffi::OsString;

use super::*;

#[test]
fn runtime_value_takes_precedence_over_other_sources() -> Result<()> {
    let value = configuration_value(
        "SETTING",
        Ok("runtime".to_owned()),
        Some("dotenv"),
        Some("embedded"),
    )?;

    assert_eq!(value, "runtime");
    Ok(())
}

#[test]
fn dotenv_value_takes_precedence_over_embedded_value() -> Result<()> {
    let value = configuration_value(
        "SETTING",
        Err(VarError::NotPresent),
        Some("dotenv"),
        Some("embedded"),
    )?;

    assert_eq!(value, "dotenv");
    Ok(())
}

#[test]
fn embedded_value_is_used_when_other_sources_are_missing() -> Result<()> {
    let value = configuration_value("SETTING", Err(VarError::NotPresent), None, Some("embedded"))?;

    assert_eq!(value, "embedded");
    Ok(())
}

#[test]
fn empty_runtime_value_is_rejected() {
    let error = configuration_value("SETTING", Ok(String::new()), None, Some("embedded"))
        .expect_err("an empty runtime override must be rejected");

    assert_eq!(error.to_string(), "SETTING must not be empty");
}

#[test]
fn empty_dotenv_value_is_rejected() {
    let error = configuration_value(
        "SETTING",
        Err(VarError::NotPresent),
        Some(""),
        Some("embedded"),
    )
    .expect_err("an empty dotenv override must be rejected");

    assert_eq!(error.to_string(), "SETTING must not be empty");
}

#[test]
fn missing_values_are_rejected() {
    let error = configuration_value("SETTING", Err(VarError::NotPresent), None, None)
        .expect_err("a missing setting must be rejected");

    assert_eq!(
        error.to_string(),
        "SETTING must be set in the environment, .env file, or embedded at build time"
    );
}

#[cfg(unix)]
#[test]
fn non_unicode_runtime_value_is_rejected() {
    use std::os::unix::ffi::OsStringExt;

    let error = configuration_value(
        "SETTING",
        Err(VarError::NotUnicode(OsString::from_vec(vec![0xff]))),
        None,
        Some("embedded"),
    )
    .expect_err("a non-Unicode runtime override must be rejected");

    assert_eq!(error.to_string(), "SETTING must contain valid Unicode");
}

#[test]
fn load_dotenv_reads_the_explicit_file() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let dotenv_path = directory.path().join(".env");
    std::fs::write(&dotenv_path, "SETTING=dotenv\n")?;

    let values = load_dotenv(&dotenv_path)?;

    assert_eq!(values.get("SETTING").map(String::as_str), Some("dotenv"));
    Ok(())
}

#[test]
fn load_dotenv_does_not_search_parent_directories() -> Result<()> {
    let parent = tempfile::tempdir()?;
    std::fs::write(parent.path().join(".env"), "SETTING=parent\n")?;
    let child = parent.path().join("child");
    std::fs::create_dir(&child)?;

    let values = load_dotenv(&child.join(".env"))?;

    assert!(values.is_empty());
    Ok(())
}
