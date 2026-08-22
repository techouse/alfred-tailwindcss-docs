use std::io::Read;
use std::time::{Duration, Instant};

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
    read_hosts: Vec<Url>,
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

#[derive(Debug)]
enum AttemptFailure {
    Retryable(anyhow::Error),
    Terminal(anyhow::Error),
}

type AttemptResult = std::result::Result<Vec<SearchResult>, AttemptFailure>;

impl AlgoliaSearch {
    /// Creates a search client using Algolia's distributed search endpoint.
    pub fn new(config: AlgoliaSearchConfig) -> Result<Self> {
        validate_config(&config)?;
        let read_hosts = read_hosts(&config.application_id)?;

        Self::with_read_hosts(config, read_hosts)
    }

    /// Searches the selected Tailwind version and returns at most 20 ranked hits.
    pub fn query(&self, query: &str, version: &str) -> Result<Vec<SearchResult>> {
        self.query_with(query, version, |endpoint, body, remaining| {
            self.request_host(endpoint, body, remaining)
        })
    }

    fn with_read_hosts(config: AlgoliaSearchConfig, read_hosts: Vec<Url>) -> Result<Self> {
        let agent = platform_agent(CONNECT_TIMEOUT, SEARCH_TIMEOUT);
        Self::with_agent(config, read_hosts, agent)
    }

    #[cfg(test)]
    fn with_read_hosts_and_agent(
        config: AlgoliaSearchConfig,
        read_hosts: Vec<Url>,
        agent: Agent,
    ) -> Result<Self> {
        Self::with_agent(config, read_hosts, agent)
    }

    fn with_agent(config: AlgoliaSearchConfig, read_hosts: Vec<Url>, agent: Agent) -> Result<Self> {
        validate_config(&config)?;
        if read_hosts.is_empty() {
            return Err(anyhow!("Algolia read hosts must not be empty"));
        }

        Ok(Self {
            config,
            read_hosts,
            agent,
        })
    }

    #[cfg(test)]
    fn with_base_url(config: AlgoliaSearchConfig, base_url: Url) -> Result<Self> {
        Self::with_read_hosts(config, vec![base_url])
    }

    fn query_with<F>(&self, query: &str, version: &str, mut attempt: F) -> Result<Vec<SearchResult>>
    where
        F: FnMut(&Url, &str, Duration) -> AttemptResult,
    {
        let body = self
            .request_body(query, version)
            .context("failed to serialize Algolia request")?;
        let deadline = Instant::now() + SEARCH_TIMEOUT;
        let mut failures = Vec::new();

        for base_url in &self.read_hosts {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                break;
            };
            if remaining.is_zero() {
                break;
            }

            let endpoint = self
                .endpoint(base_url)
                .context("failed to construct Algolia endpoint")?;

            match attempt(&endpoint, &body, remaining) {
                Ok(hits) => return Ok(hits),
                Err(AttemptFailure::Retryable(error)) => failures.push(error.to_string()),
                Err(AttemptFailure::Terminal(error)) => return Err(error),
            }
        }

        if failures.is_empty() {
            Err(anyhow!(
                "Algolia request timed out before a read host could be tried"
            ))
        } else {
            Err(anyhow!(
                "Algolia request failed after trying {} read hosts: {}",
                failures.len(),
                failures.join("; ")
            ))
        }
    }

    fn request_host(&self, endpoint: &Url, body: &str, remaining: Duration) -> AttemptResult {
        let request = self
            .agent
            .post(endpoint.as_str())
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("x-algolia-application-id", &self.config.application_id)
            .header("x-algolia-api-key", &self.config.api_key)
            .config()
            .http_status_as_error(false)
            .timeout_global(Some(remaining))
            .timeout_connect(Some(CONNECT_TIMEOUT.min(remaining)))
            .build();
        let mut response = request
            .send(body)
            .map_err(|error| classify_ureq_error(endpoint, "send", error))?;
        let status = response.status().as_u16();
        let mut response_reader = response
            .body_mut()
            .with_config()
            .limit(MAX_RESPONSE_BYTES + 1)
            .reader()
            .take(MAX_RESPONSE_BYTES + 1);
        let mut response_body = String::new();
        response_reader
            .read_to_string(&mut response_body)
            .map_err(|error| {
                classify_ureq_error(endpoint, "read response body", ureq::Error::from(error))
            })?;
        if response_body.len() as u64 > MAX_RESPONSE_BYTES {
            let error =
                anyhow!("Algolia response body from {endpoint} exceeds {MAX_RESPONSE_BYTES} bytes");
            return Err(if (500..=599).contains(&status) {
                AttemptFailure::Retryable(error)
            } else {
                AttemptFailure::Terminal(error)
            });
        }

        if (200..=299).contains(&status) {
            let response: SearchResponse =
                serde_json::from_str(&response_body).map_err(|error| {
                    AttemptFailure::Terminal(anyhow!(
                        "failed to deserialize Algolia response from {endpoint}: {error}"
                    ))
                })?;
            return Ok(response.hits);
        }

        let error = anyhow!(
            "Algolia request to {endpoint} failed with HTTP status {status}: {response_body}"
        );
        if (400..=499).contains(&status) {
            Err(AttemptFailure::Terminal(error))
        } else if (500..=599).contains(&status) {
            Err(AttemptFailure::Retryable(error))
        } else {
            Err(AttemptFailure::Terminal(error))
        }
    }

    fn endpoint(&self, base_url: &Url) -> Result<Url> {
        let mut endpoint = base_url.clone();
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

fn read_hosts(application_id: &str) -> Result<Vec<Url>> {
    [
        format!("https://{application_id}-dsn.algolia.net/"),
        format!("https://{application_id}-1.algolianet.com/"),
        format!("https://{application_id}-2.algolianet.com/"),
        format!("https://{application_id}-3.algolianet.com/"),
    ]
    .into_iter()
    .map(|host| Url::parse(&host).with_context(|| format!("invalid Algolia host {host}")))
    .collect()
}

fn classify_ureq_error(endpoint: &Url, phase: &str, error: ureq::Error) -> AttemptFailure {
    let retryable = matches!(
        &error,
        ureq::Error::Timeout(_)
            | ureq::Error::Io(_)
            | ureq::Error::Protocol(_)
            | ureq::Error::HostNotFound
            | ureq::Error::ConnectionFailed
    );
    let error = anyhow!("Algolia {phase} failed for {endpoint}: {error}");

    if retryable {
        AttemptFailure::Retryable(error)
    } else {
        AttemptFailure::Terminal(error)
    }
}

fn validate_config(config: &AlgoliaSearchConfig) -> Result<()> {
    if config.application_id.is_empty() {
        return Err(anyhow!("ALGOLIA_APPLICATION_ID must not be empty"));
    }
    if config.api_key.is_empty() {
        return Err(anyhow!("ALGOLIA_SEARCH_ONLY_API_KEY must not be empty"));
    }
    if config.index_name.is_empty() {
        return Err(anyhow!("ALGOLIA_SEARCH_INDEX must not be empty"));
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/algolia.rs"]
mod tests;
