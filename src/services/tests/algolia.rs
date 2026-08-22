use super::*;
use flate2::{Compression, write::GzEncoder};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use ureq::Agent;
use ureq::tls::{RootCerts, TlsConfig};

use serde_json::{Value, json};

fn config() -> AlgoliaSearchConfig {
    AlgoliaSearchConfig {
        application_id: "app".to_owned(),
        api_key: "key".to_owned(),
        index_name: "tailwind".to_owned(),
    }
}

#[test]
fn endpoint_uses_single_index_search_route() -> Result<()> {
    let base_url = Url::parse("http://127.0.0.1:8080/api/")?;
    let client = AlgoliaSearch::with_base_url(config(), base_url.clone())?;

    assert_eq!(
        client.endpoint(&base_url)?.as_str(),
        "http://127.0.0.1:8080/api/1/indexes/tailwind/query"
    );
    Ok(())
}

#[test]
fn client_uses_dsn_and_numbered_read_hosts_in_order() -> Result<()> {
    let client = AlgoliaSearch::new(config())?;

    assert_eq!(
        client
            .read_hosts
            .iter()
            .map(Url::as_str)
            .collect::<Vec<_>>(),
        vec![
            "https://app-dsn.algolia.net/",
            "https://app-1.algolianet.com/",
            "https://app-2.algolianet.com/",
            "https://app-3.algolianet.com/",
        ]
    );
    Ok(())
}

#[test]
fn client_uses_platform_verifier_and_search_timeouts() -> Result<()> {
    let client = AlgoliaSearch::with_base_url(config(), Url::parse("http://localhost/")?)?;
    let timeouts = client.agent.config().timeouts();

    assert!(matches!(
        client.agent.config().tls_config().root_certs(),
        RootCerts::PlatformVerifier
    ));
    assert_eq!(timeouts.connect, Some(CONNECT_TIMEOUT));
    assert_eq!(timeouts.global, Some(SEARCH_TIMEOUT));
    Ok(())
}

#[test]
fn request_body_preserves_tailwind_search_contract() -> Result<()> {
    let client = AlgoliaSearch::with_base_url(config(), Url::parse("http://localhost/")?)?;
    let body: Value = serde_json::from_str(&client.request_body("background color", "v4")?)?;

    assert_eq!(
        body,
        json!({
            "query": "background color",
            "facetFilters": ["version:v4"],
            "attributesToRetrieve": [
                "hierarchy.lvl0", "hierarchy.lvl1", "hierarchy.lvl2",
                "hierarchy.lvl3", "hierarchy.lvl4", "hierarchy.lvl5",
                "hierarchy.lvl6", "content", "type", "url"
            ],
            "attributesToSnippet": [
                "hierarchy.lvl1:10", "hierarchy.lvl2:10", "hierarchy.lvl3:10",
                "hierarchy.lvl4:10", "hierarchy.lvl5:10", "hierarchy.lvl6:10",
                "content:10"
            ],
            "snippetEllipsisText": "...",
            "distinct": 1,
            "page": 0,
            "hitsPerPage": 20
        })
    );
    Ok(())
}

#[test]
fn query_retries_retryable_failures_in_host_order() -> Result<()> {
    let client = client_with_hosts(&[
        "http://first.test/",
        "http://second.test/",
        "http://third.test/",
    ])?;
    let mut endpoints = Vec::new();

    let hits = client.query_with("background", "v4", |endpoint, _, _| {
        endpoints.push(endpoint.as_str().to_owned());
        if endpoints.len() < 3 {
            Err(AttemptFailure::Retryable(anyhow::anyhow!(
                "temporary failure"
            )))
        } else {
            Ok(Vec::new())
        }
    })?;

    assert!(hits.is_empty());
    assert_eq!(
        endpoints,
        vec![
            "http://first.test/1/indexes/tailwind/query",
            "http://second.test/1/indexes/tailwind/query",
            "http://third.test/1/indexes/tailwind/query",
        ]
    );
    Ok(())
}

#[test]
fn query_stops_after_terminal_failure() -> Result<()> {
    let client = client_with_hosts(&["http://first.test/", "http://second.test/"])?;
    let mut attempts = 0;

    let error = client
        .query_with("background", "v4", |_, _, _| {
            attempts += 1;
            Err(AttemptFailure::Terminal(anyhow::anyhow!("invalid request")))
        })
        .expect_err("terminal failures must stop failover");

    assert_eq!(attempts, 1);
    assert_eq!(error.to_string(), "invalid request");
    Ok(())
}

#[test]
fn query_reports_all_retryable_host_failures() -> Result<()> {
    let client = client_with_hosts(&["http://first.test/", "http://second.test/"])?;
    let mut attempts = 0;

    let error = client
        .query_with("background", "v4", |endpoint, _, _| {
            attempts += 1;
            Err(AttemptFailure::Retryable(anyhow::anyhow!(
                "{} failed",
                endpoint
            )))
        })
        .expect_err("all retryable failures must be reported");
    let message = error.to_string();

    assert_eq!(attempts, 2);
    assert!(message.contains("after trying 2 read hosts"));
    assert!(message.contains("first.test/1/indexes/tailwind/query failed"));
    assert!(message.contains("second.test/1/indexes/tailwind/query failed"));
    Ok(())
}

#[test]
fn query_uses_one_shared_timeout_budget() -> Result<()> {
    let client = client_with_hosts(&[
        "http://first.test/",
        "http://second.test/",
        "http://third.test/",
        "http://fourth.test/",
    ])?;
    let mut remaining_times = Vec::new();

    let _ = client.query_with("background", "v4", |_, _, remaining| {
        remaining_times.push(remaining);
        thread::sleep(Duration::from_millis(5));
        Err(AttemptFailure::Retryable(anyhow::anyhow!(
            "temporary failure"
        )))
    });

    assert_eq!(remaining_times.len(), 4);
    assert!(remaining_times[0] <= SEARCH_TIMEOUT);
    assert!(
        remaining_times
            .windows(2)
            .all(|window| window[0] > window[1])
    );
    Ok(())
}

#[test]
fn transport_failure_classification_matches_retry_policy() -> Result<()> {
    let endpoint = Url::parse("http://first.test/")?;

    assert!(matches!(
        classify_ureq_error(&endpoint, "send", ureq::Error::HostNotFound),
        AttemptFailure::Retryable(_)
    ));
    assert!(matches!(
        classify_ureq_error(&endpoint, "send", ureq::Error::ConnectionFailed),
        AttemptFailure::Retryable(_)
    ));
    assert!(matches!(
        classify_ureq_error(
            &endpoint,
            "send",
            ureq::Error::Timeout(ureq::Timeout::Global)
        ),
        AttemptFailure::Retryable(_)
    ));
    assert!(matches!(
        classify_ureq_error(
            &endpoint,
            "send",
            ureq::Error::Io(std::io::Error::other("connection reset"))
        ),
        AttemptFailure::Retryable(_)
    ));
    assert!(matches!(
        classify_ureq_error(
            &endpoint,
            "send",
            ureq::Error::BodyExceedsLimit(MAX_RESPONSE_BYTES + 1)
        ),
        AttemptFailure::Terminal(_)
    ));
    assert!(matches!(
        classify_ureq_error(
            &endpoint,
            "read response body",
            ureq::Error::from(ureq::Error::BodyExceedsLimit(1).into_io())
        ),
        AttemptFailure::Terminal(_)
    ));
    Ok(())
}

#[test]
fn search_response_deserializes_hierarchy() -> Result<()> {
    let response: SearchResponse = serde_json::from_value(json!({
        "hits": [{
            "objectID": "background-color",
            "type": "lvl1",
            "url": "https://tailwindcss.com/docs/background-color",
            "hierarchy": {
                "lvl0": "Docs",
                "lvl1": "Background Color",
                "lvl2": null,
                "lvl3": null,
                "lvl4": null,
                "lvl5": null,
                "lvl6": null
            },
            "content": null
        }]
    }))?;

    assert_eq!(
        response.hits[0].hierarchy.level(1),
        Some("Background Color")
    );
    Ok(())
}

#[test]
fn search_response_deserializes_content_result() -> Result<()> {
    let response: SearchResponse = serde_json::from_value(json!({
        "hits": [{
            "objectID": "utility-first",
            "type": "content",
            "url": "https://tailwindcss.com/docs/styling-with-utility-classes",
            "hierarchy": {
                "lvl0": "Core concepts",
                "lvl1": "Styling with utility classes",
                "lvl2": null,
                "lvl3": null,
                "lvl4": null,
                "lvl5": null,
                "lvl6": null
            },
            "content": "Building complex components from a constrained set of primitive utilities."
        }]
    }))?;

    assert_eq!(response.hits[0].hierarchy_level()?, 0);
    Ok(())
}

#[test]
fn empty_configuration_values_are_rejected_before_client_creation() {
    for (field, expected) in [
        ("application_id", "ALGOLIA_APPLICATION_ID must not be empty"),
        ("api_key", "ALGOLIA_SEARCH_ONLY_API_KEY must not be empty"),
        ("index_name", "ALGOLIA_SEARCH_INDEX must not be empty"),
    ] {
        let mut invalid = config();
        match field {
            "application_id" => invalid.application_id.clear(),
            "api_key" => invalid.api_key.clear(),
            "index_name" => invalid.index_name.clear(),
            _ => unreachable!("test field is fixed"),
        }

        let error = match AlgoliaSearch::new(invalid) {
            Ok(_) => panic!("empty values must fail"),
            Err(error) => error,
        };
        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn query_falls_back_after_5xx_response() -> Result<()> {
    let (first_url, first_server) = serve_once(503, r#"{"message":"temporary"}"#)?;
    let (second_url, second_server) = serve_once(200, r#"{"hits":[]}"#)?;
    let client = AlgoliaSearch::with_read_hosts_and_agent(
        config(),
        vec![first_url, second_url],
        no_proxy_agent(),
    )?;

    let hits = client.query("background", "v4")?;

    first_server
        .join()
        .map_err(|_| anyhow::anyhow!("first server thread panicked"))?;
    second_server
        .join()
        .map_err(|_| anyhow::anyhow!("second server thread panicked"))?;
    assert!(hits.is_empty());
    Ok(())
}

#[test]
fn query_includes_4xx_response_body() -> Result<()> {
    let (url, server) = serve_once(401, r#"{"message":"invalid key"}"#)?;
    let client = AlgoliaSearch::with_read_hosts_and_agent(config(), vec![url], no_proxy_agent())?;

    let error = client
        .query("background", "v4")
        .expect_err("4xx responses must fail");
    server
        .join()
        .map_err(|_| anyhow::anyhow!("server thread panicked"))?;
    let message = error.to_string();

    assert!(message.contains("HTTP status 401"));
    assert!(message.contains(r#"{"message":"invalid key"}"#));
    Ok(())
}

#[test]
fn query_includes_5xx_response_body_after_hosts_fail() -> Result<()> {
    let (url, server) = serve_once(503, r#"{"message":"service unavailable"}"#)?;
    let client = AlgoliaSearch::with_read_hosts_and_agent(config(), vec![url], no_proxy_agent())?;

    let error = client
        .query("background", "v4")
        .expect_err("5xx responses must fail after exhausting hosts");
    server
        .join()
        .map_err(|_| anyhow::anyhow!("server thread panicked"))?;
    let message = error.to_string();

    assert!(message.contains("HTTP status 503"));
    assert!(message.contains(r#"{"message":"service unavailable"}"#));
    Ok(())
}

#[test]
fn query_rejects_oversized_decoded_response() -> Result<()> {
    let body = "x".repeat((MAX_RESPONSE_BYTES + 1) as usize);
    let (url, server) = serve_gzip_once(200, &body)?;
    let client = AlgoliaSearch::with_read_hosts_and_agent(config(), vec![url], no_proxy_agent())?;

    let error = client
        .query("background", "v4")
        .expect_err("oversized decoded responses must fail");
    server
        .join()
        .map_err(|_| anyhow::anyhow!("server thread panicked"))?;

    assert!(error.to_string().contains("exceeds"));
    Ok(())
}

#[test]
fn query_falls_back_after_oversized_5xx_response() -> Result<()> {
    let body = "x".repeat((MAX_RESPONSE_BYTES + 1) as usize);
    let (first_url, first_server) = serve_gzip_once(503, &body)?;
    let (second_url, second_server) = serve_once(200, r#"{"hits":[]}"#)?;
    let client = AlgoliaSearch::with_read_hosts_and_agent(
        config(),
        vec![first_url, second_url],
        no_proxy_agent(),
    )?;

    let hits = client.query("background", "v4")?;

    first_server
        .join()
        .map_err(|_| anyhow::anyhow!("first server thread panicked"))?;
    second_server
        .join()
        .map_err(|_| anyhow::anyhow!("second server thread panicked"))?;

    assert!(hits.is_empty());
    Ok(())
}

#[test]
fn query_accepts_response_exactly_at_size_cap() -> Result<()> {
    let prefix = "{\"hits\":[],\"pad\":\"";
    let suffix = "\"}";
    let pad_length = MAX_RESPONSE_BYTES as usize - prefix.len() - suffix.len();
    let body = format!("{prefix}{}{suffix}", "x".repeat(pad_length));
    assert_eq!(body.len() as u64, MAX_RESPONSE_BYTES);
    let (url, server) = serve_once(200, &body)?;
    let client = AlgoliaSearch::with_read_hosts_and_agent(config(), vec![url], no_proxy_agent())?;

    let hits = client.query("background", "v4")?;

    server
        .join()
        .map_err(|_| anyhow::anyhow!("server thread panicked"))?;
    assert!(hits.is_empty());
    Ok(())
}

#[test]
fn query_falls_back_after_malformed_http_response() -> Result<()> {
    let (first_url, first_server) = serve_raw_once(b"BOGUS/1.1 200\r\n\r\n")?;
    let (second_url, second_server) = serve_once(200, r#"{"hits":[]}"#)?;
    let client = AlgoliaSearch::with_read_hosts_and_agent(
        config(),
        vec![first_url, second_url],
        no_proxy_agent(),
    )?;

    let hits = client.query("background", "v4")?;

    first_server
        .join()
        .map_err(|_| anyhow::anyhow!("first server thread panicked"))?;
    second_server
        .join()
        .map_err(|_| anyhow::anyhow!("second server thread panicked"))?;
    assert!(hits.is_empty());
    Ok(())
}

fn client_with_hosts(hosts: &[&str]) -> Result<AlgoliaSearch> {
    let read_hosts = hosts
        .iter()
        .map(|host| Url::parse(host))
        .collect::<std::result::Result<Vec<_>, _>>()?;

    AlgoliaSearch::with_read_hosts_and_agent(config(), read_hosts, no_proxy_agent())
}

fn no_proxy_agent() -> Agent {
    Agent::config_builder()
        .proxy(None)
        .tls_config(
            TlsConfig::builder()
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_global(Some(SEARCH_TIMEOUT))
        .build()
        .into()
}

fn serve_once(status: u16, body: &str) -> Result<(Url, JoinHandle<()>)> {
    serve_once_with_body(status, body.as_bytes().to_vec(), None)
}

fn serve_gzip_once(status: u16, body: &str) -> Result<(Url, JoinHandle<()>)> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(body.as_bytes())?;
    serve_once_with_body(status, encoder.finish()?, Some("gzip"))
}

fn serve_raw_once(raw: &'static [u8]) -> Result<(Url, JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let server = thread::spawn(move || {
        let (mut stream, _) = listener
            .accept()
            .expect("test server must accept a request");
        read_request(&mut stream);
        stream
            .write_all(raw)
            .expect("test server must write the raw response");
        stream.flush().expect("test server must flush the response");
    });

    Ok((Url::parse(&format!("http://{address}/"))?, server))
}

fn read_request(stream: &mut TcpStream) {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        let bytes = stream
            .read(&mut buffer)
            .expect("test server must read the request");
        if bytes == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..bytes]);
        let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
            continue;
        };
        let headers = String::from_utf8_lossy(&request[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("Content-Length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            })
            .unwrap_or(0);
        if request.len() >= header_end + 4 + content_length {
            break;
        }
    }
}

fn serve_once_with_body(
    status: u16,
    body: Vec<u8>,
    content_encoding: Option<&str>,
) -> Result<(Url, JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let content_encoding = content_encoding.map(str::to_owned).unwrap_or_default();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener
            .accept()
            .expect("test server must accept a request");
        read_request(&mut stream);
        let encoding_header = if content_encoding.is_empty() {
            String::new()
        } else {
            format!("Content-Encoding: {content_encoding}\r\n")
        };
        write!(
            stream,
            "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{encoding_header}Connection: close\r\n\r\n",
            body.len()
        )
        .expect("test server must write the response headers");
        stream
            .write_all(&body)
            .expect("test server must write the response body");
        stream.flush().expect("test server must flush the response");
    });

    Ok((Url::parse(&format!("http://{address}/"))?, server))
}
