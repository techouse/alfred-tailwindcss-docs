use anyhow::{Result, bail};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Cli {
    pub(crate) query: String,
    pub(crate) verbose: bool,
    pub(crate) update: bool,
}

impl Cli {
    pub(crate) fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut cli = Self::default();
        let mut arguments = arguments.into_iter();

        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "-q" | "--query" => {
                    cli.query = arguments
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("{argument} requires a value"))?;
                }
                "-v" | "--verbose" => cli.verbose = true,
                "-u" | "--update" => cli.update = true,
                _ if argument.starts_with("--query=") => {
                    cli.query = argument["--query=".len()..].to_owned();
                }
                _ => bail!("unknown argument: {argument}"),
            }
        }

        Ok(cli)
    }

    pub(crate) fn normalized_query(&self, tailwind_version: &str) -> String {
        self.query
            .split_whitespace()
            .filter(|part| *part != tailwind_version)
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    }
}

#[cfg(test)]
#[path = "tests/cli.rs"]
mod tests;
