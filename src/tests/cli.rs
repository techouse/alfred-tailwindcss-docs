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
fn parse_rejects_unknown_arguments() {
    let error = Cli::parse(["--unknown".to_owned()]).expect_err("unknown flag must fail");

    assert_eq!(error.to_string(), "unknown argument: --unknown");
}
