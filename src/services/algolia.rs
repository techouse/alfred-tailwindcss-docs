use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use serde::Serialize;
use ureq::Agent;
use url::Url;

use crate::models::{SearchResponse, SearchResult};

use super::http::platform_agent;

const MAX_RESPONSE_BYTES: u64 = 2 * 1024 * 1024;
const SEARCH_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const ATTRIBUTES_TO_RETRIEVE: &[&str] = &[
    "hierarchy.lvl0",
    "hierarchy.lvl1",
    "hierarchy.lvl2",
    "hierarchy.lvl3",
    "hierarchy.lvl4",
    "hierarchy.lvl5",
    "hierarchy.lvl6",
    "content",
    "type",
    "url",
];
const ATTRIBUTES_TO_SNIPPET: &[&str] = &[
    "hierarchy.lvl1:10",
    "hierarchy.lvl2:10",
    "hierarchy.lvl3:10",
    "hierarchy.lvl4:10",
    "hierarchy.lvl5:10",
    "hierarchy.lvl6:10",
    "content:10",
];

/// Configuration required to query the Tailwind CSS Algolia index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlgoliaSearchConfig {
    /// Algolia application identifier.
    pub application_id: String,
    /// Search-only Algolia API key.
    pub api_key: String,
    /// Algolia index name.
    pub index_name: String,
}

/// Synchronous client for the Tailwind CSS Algolia index.
#[derive(Clone)]
pub struct AlgoliaSearch {
    config: AlgoliaSearchConfig,
    base_url: Url,
    agent: Agent,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchRequest<'a> {
    query: &'a str,
    facet_filters: [String; 1],
    attributes_to_retrieve: &'static [&'static str],
    attributes_to_snippet: &'static [&'static str],
    snippet_ellipsis_text: &'static str,
    distinct: u8,
    page: u8,
    hits_per_page: u8,
}

impl AlgoliaSearch {
    /// Creates a search client using Algolia's distributed search endpoint.
    pub fn new(config: AlgoliaSearchConfig) -> Result<Self> {
        let base_url = Url::parse(&format!(
            "https://{}-dsn.algolia.net/",
            config.application_id
        ))
        .context("invalid Algolia application identifier")?;

        Self::with_base_url(config, base_url)
    }

    /// Searches the selected Tailwind version and returns at most 20 ranked hits.
    pub fn query(&self, query: &str, version: &str) -> Result<Vec<SearchResult>> {
        let endpoint = self.endpoint()?;
        let body = self.request_body(query, version)?;
        let mut response = self
            .agent
            .post(endpoint.as_str())
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("x-algolia-application-id", &self.config.application_id)
            .header("x-algolia-api-key", &self.config.api_key)
            .send(&body)
            .map_err(|error| anyhow!("Algolia request failed: {error}"))?;
        let response_body = response
            .body_mut()
            .with_config()
            .limit(MAX_RESPONSE_BYTES)
            .read_to_string()
            .map_err(|error| anyhow!("failed to read Algolia response: {error}"))?;
        let response: SearchResponse =
            serde_json::from_str(&response_body).context("invalid Algolia response JSON")?;

        Ok(response.hits)
    }

    fn with_base_url(config: AlgoliaSearchConfig, base_url: Url) -> Result<Self> {
        if config.application_id.is_empty() {
            return Err(anyhow!("ALGOLIA_APPLICATION_ID must not be empty"));
        }
        if config.api_key.is_empty() {
            return Err(anyhow!("ALGOLIA_SEARCH_ONLY_API_KEY must not be empty"));
        }
        if config.index_name.is_empty() {
            return Err(anyhow!("ALGOLIA_SEARCH_INDEX must not be empty"));
        }

        let agent = platform_agent(CONNECT_TIMEOUT, SEARCH_TIMEOUT);

        Ok(Self {
            config,
            base_url,
            agent,
        })
    }

    fn endpoint(&self) -> Result<Url> {
        let mut endpoint = self.base_url.clone();
        let mut segments = endpoint
            .path_segments_mut()
            .map_err(|()| anyhow!("Algolia base URL cannot be a base"))?;
        segments.pop_if_empty();
        segments.extend(["1", "indexes", self.config.index_name.as_str(), "query"]);
        drop(segments);
        Ok(endpoint)
    }

    fn request_body(&self, query: &str, version: &str) -> Result<String> {
        Ok(serde_json::to_string(&SearchRequest {
            query,
            facet_filters: [format!("version:{version}")],
            attributes_to_retrieve: ATTRIBUTES_TO_RETRIEVE,
            attributes_to_snippet: ATTRIBUTES_TO_SNIPPET,
            snippet_ellipsis_text: "...",
            distinct: 1,
            page: 0,
            hits_per_page: 20,
        })?)
    }
}

#[cfg(test)]
#[path = "tests/algolia.rs"]
mod tests;
