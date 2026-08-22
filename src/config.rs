use std::collections::HashMap;
use std::env::VarError;
use std::path::Path;

use alfred_tailwindcss_docs::services::AlgoliaSearchConfig;
use anyhow::{Context, Result, anyhow};

const EMBEDDED_APPLICATION_ID: Option<&str> = option_env!("ALGOLIA_APPLICATION_ID");
const EMBEDDED_SEARCH_ONLY_API_KEY: Option<&str> = option_env!("ALGOLIA_SEARCH_ONLY_API_KEY");
const EMBEDDED_SEARCH_INDEX: Option<&str> = option_env!("ALGOLIA_SEARCH_INDEX");

pub(crate) fn algolia_search_config() -> Result<AlgoliaSearchConfig> {
    algolia_search_config_from(
        std::env::var("ALGOLIA_APPLICATION_ID"),
        std::env::var("ALGOLIA_SEARCH_ONLY_API_KEY"),
        std::env::var("ALGOLIA_SEARCH_INDEX"),
        Path::new(".env"),
    )
}

fn algolia_search_config_from(
    runtime_application_id: Result<String, VarError>,
    runtime_api_key: Result<String, VarError>,
    runtime_index_name: Result<String, VarError>,
    dotenv_path: &Path,
) -> Result<AlgoliaSearchConfig> {
    let dotenv_values = if matches!(&runtime_application_id, Err(VarError::NotPresent))
        || matches!(&runtime_api_key, Err(VarError::NotPresent))
        || matches!(&runtime_index_name, Err(VarError::NotPresent))
    {
        load_dotenv(dotenv_path)?
    } else {
        HashMap::new()
    };

    Ok(AlgoliaSearchConfig {
        application_id: configuration_value(
            "ALGOLIA_APPLICATION_ID",
            runtime_application_id,
            dotenv_values
                .get("ALGOLIA_APPLICATION_ID")
                .map(String::as_str),
            EMBEDDED_APPLICATION_ID,
        )?,
        api_key: configuration_value(
            "ALGOLIA_SEARCH_ONLY_API_KEY",
            runtime_api_key,
            dotenv_values
                .get("ALGOLIA_SEARCH_ONLY_API_KEY")
                .map(String::as_str),
            EMBEDDED_SEARCH_ONLY_API_KEY,
        )?,
        index_name: configuration_value(
            "ALGOLIA_SEARCH_INDEX",
            runtime_index_name,
            dotenv_values
                .get("ALGOLIA_SEARCH_INDEX")
                .map(String::as_str),
            EMBEDDED_SEARCH_INDEX,
        )?,
    })
}

fn load_dotenv(path: &Path) -> Result<HashMap<String, String>> {
    let entries = match dotenvy::from_path_iter(path) {
        Ok(entries) => entries,
        Err(error) if error.not_found() => return Ok(HashMap::new()),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", path.display()));
        }
    };

    let mut values = HashMap::new();
    for entry in entries {
        let (key, value) = entry.with_context(|| format!("failed to parse {}", path.display()))?;
        values.entry(key).or_insert(value);
    }

    Ok(values)
}

fn configuration_value(
    name: &str,
    runtime_value: Result<String, VarError>,
    dotenv_value: Option<&str>,
    embedded_value: Option<&str>,
) -> Result<String> {
    match runtime_value {
        Ok(value) if value.is_empty() => Err(anyhow!("{name} must not be empty")),
        Ok(value) => Ok(value),
        Err(VarError::NotUnicode(_)) => Err(anyhow!("{name} must contain valid Unicode")),
        Err(VarError::NotPresent) => match dotenv_value {
            Some("") => Err(anyhow!("{name} must not be empty")),
            Some(value) => Ok(value.to_owned()),
            None => match embedded_value {
                Some("") => Err(anyhow!("{name} must not be empty")),
                Some(value) => Ok(value.to_owned()),
                None => Err(anyhow!(
                    "{name} must be set in the environment, .env file, or embedded at build time"
                )),
            },
        },
    }
}

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;
