use super::*;

#[test]
fn parse_reads_query_and_flags() -> Result<()> {
    let cli = Cli::parse([
        "-q".to_owned(),
        "  v4   Background Color ".to_owned(),
        "--verbose".to_owned(),
    ])?;

    assert_eq!(
        cli,
        Cli {
            query: "  v4   Background Color ".to_owned(),
            verbose: true,
            update: false,
        }
    );
    Ok(())
}

#[test]
fn normalized_query_removes_selected_version_and_collapses_whitespace() {
    let cli = Cli {
        query: "  v4   Background Color v4 ".to_owned(),
        ..Cli::default()
    };

    assert_eq!(cli.normalized_query("v4"), "background color");
}

#[test]
fn parse_rejects_missing_query_value() {
    let error = Cli::parse(["--query".to_owned()]).expect_err("query value must be required");

    assert_eq!(error.to_string(), "--query requires a value");
}

#[test]
fn parse_rejects_missing_short_query_value() {
    let error = Cli::parse(["-q".to_owned()]).expect_err("query value must be required");

    assert_eq!(error.to_string(), "-q requires a value");
}

#[test]
fn parse_rejects_long_query_followed_by_verbose_flag() {
    let error = Cli::parse(["--query".to_owned(), "--verbose".to_owned()])
        .expect_err("query value must be required");

    assert_eq!(error.to_string(), "--query requires a value");
}

#[test]
fn parse_rejects_short_query_followed_by_update_flag() {
    let error =
        Cli::parse(["-q".to_owned(), "-u".to_owned()]).expect_err("query value must be required");

    assert_eq!(error.to_string(), "-q requires a value");
}

#[test]
fn parse_accepts_unrecognized_dash_prefixed_query() -> Result<()> {
    let cli = Cli::parse(["-q".to_owned(), "--force".to_owned()])?;

    assert_eq!(cli.query, "--force");
    Ok(())
}

#[test]
fn parse_accepts_recognized_flag_text_with_equals_query() -> Result<()> {
    let cli = Cli::parse(["--query=-u".to_owned()])?;

    assert_eq!(cli.query, "-u");
    Ok(())
}

#[test]
fn parse_accepts_attached_short_query_value() -> Result<()> {
    let cli = Cli::parse(["-qbackground".to_owned()])?;

    assert_eq!(cli.query, "background");
    Ok(())
}

#[test]
fn parse_accepts_collapsed_short_flags() -> Result<()> {
    let cli = Cli::parse(["-vu".to_owned()])?;

    assert_eq!(
        cli,
        Cli {
            query: String::new(),
            verbose: true,
            update: true,
        }
    );
    Ok(())
}

#[test]
fn parse_accepts_collapsed_flags_with_attached_query() -> Result<()> {
    let cli = Cli::parse(["-vuqbackground".to_owned()])?;

    assert_eq!(
        cli,
        Cli {
            query: "background".to_owned(),
            verbose: true,
            update: true,
        }
    );
    Ok(())
}

#[test]
fn parse_accepts_collapsed_flags_with_separated_query() -> Result<()> {
    let cli = Cli::parse(["-vq".to_owned(), "background".to_owned()])?;

    assert_eq!(
        cli,
        Cli {
            query: "background".to_owned(),
            verbose: true,
            update: false,
        }
    );
    Ok(())
}

#[test]
fn parse_rejects_unknown_flag_in_cluster() {
    let error = Cli::parse(["-vx".to_owned()]).expect_err("unknown cluster flag must fail");

    assert_eq!(error.to_string(), "unknown argument: -vx");
}

#[test]
fn parse_rejects_cluster_query_without_value() {
    let error = Cli::parse(["-vq".to_owned()]).expect_err("cluster query value must be required");

    assert_eq!(error.to_string(), "-q requires a value");
}

#[test]
fn parse_rejects_unknown_arguments() {
    let error = Cli::parse(["--unknown".to_owned()]).expect_err("unknown flag must fail");

    assert_eq!(error.to_string(), "unknown argument: --unknown");
}
