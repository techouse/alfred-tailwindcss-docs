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
                    cli.query = take_query_value(&argument, &mut arguments)?;
                }
                "-v" | "--verbose" => cli.verbose = true,
                "-u" | "--update" => cli.update = true,
                _ if argument.starts_with("--query=") => {
                    cli.query = argument["--query=".len()..].to_owned();
                }
                _ if is_valid_short_option_cluster(&argument) => {
                    for (offset, character) in argument[1..].char_indices() {
                        match character {
                            'v' => cli.verbose = true,
                            'u' => cli.update = true,
                            'q' => {
                                let attached = &argument[1 + offset + 1..];
                                cli.query = if attached.is_empty() {
                                    take_query_value("-q", &mut arguments)?
                                } else {
                                    attached.to_owned()
                                };
                                break;
                            }
                            _ => bail!("unknown argument: {argument}"),
                        }
                    }
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

fn take_query_value(option: &str, arguments: &mut impl Iterator<Item = String>) -> Result<String> {
    let value = arguments
        .next()
        .ok_or_else(|| anyhow::anyhow!("{option} requires a value"))?;
    if matches!(
        value.as_str(),
        "-q" | "--query" | "-v" | "--verbose" | "-u" | "--update"
    ) || value.starts_with("--query=")
        || is_valid_short_option_cluster(&value)
    {
        bail!("{option} requires a value");
    }

    Ok(value)
}

fn is_valid_short_option_cluster(argument: &str) -> bool {
    if argument.len() <= 2 || !argument.starts_with('-') || argument.starts_with("--") {
        return false;
    }

    for character in argument[1..].chars() {
        match character {
            'v' | 'u' => {}
            'q' => return true,
            _ => return false,
        }
    }

    true
}

#[cfg(test)]
#[path = "tests/cli.rs"]
mod tests;
