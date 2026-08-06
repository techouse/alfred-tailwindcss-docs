use serde_json::json;

use super::*;

fn config() -> AlgoliaSearchConfig {
    AlgoliaSearchConfig {
        application_id: "app".to_owned(),
        api_key: "key".to_owned(),
        index_name: "tailwind".to_owned(),
    }
}

#[test]
fn endpoint_uses_single_index_search_route() -> Result<()> {
    let client = AlgoliaSearch::with_base_url(config(), Url::parse("http://127.0.0.1:8080/api/")?)?;

    assert_eq!(
        client.endpoint()?.as_str(),
        "http://127.0.0.1:8080/api/1/indexes/tailwind/query"
    );
    Ok(())
}

#[test]
fn request_body_preserves_tailwind_search_contract() -> Result<()> {
    let client = AlgoliaSearch::with_base_url(config(), Url::parse("http://localhost/")?)?;
    let body: serde_json::Value =
        serde_json::from_str(&client.request_body("background color", "v4")?)?;

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
