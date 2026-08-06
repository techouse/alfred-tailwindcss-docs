use super::*;
use crate::models::SearchResultHierarchy;

fn result(result_type: &str) -> SearchResult {
    SearchResult {
        object_id: "background-color".to_owned(),
        result_type: result_type.to_owned(),
        url: "https://tailwindcss.com/docs/background-color".to_owned(),
        hierarchy: SearchResultHierarchy {
            lvl0: "Docs &amp; Guides".to_owned(),
            lvl1: Some("Background &amp; Color".to_owned()),
            lvl2: None,
            lvl3: None,
            lvl4: None,
            lvl5: None,
            lvl6: None,
        },
        content: None,
    }
}

#[test]
fn items_from_results_preserves_provider_order() -> Result<()> {
    let mut second = result("content");
    second.object_id = "second".to_owned();

    let items = items_from_results(&[result("lvl1"), second])?;

    assert_eq!(
        items.iter().map(Item::uid).collect::<Vec<_>>(),
        vec![Some("background-color"), Some("second")]
    );
    Ok(())
}

#[test]
fn content_item_uses_root_title_without_breadcrumb() -> Result<()> {
    let items = items_from_results(&[result("content")])?;

    assert_eq!(
        (items[0].title(), items[0].subtitle()),
        ("Docs &amp; Guides", None)
    );
    Ok(())
}

#[test]
fn item_uses_selected_hierarchy_level_as_title() -> Result<()> {
    let items = items_from_results(&[result("lvl1")])?;

    assert_eq!(items[0].title(), "Background &amp; Color");
    Ok(())
}

#[test]
fn item_decodes_html_entities_in_breadcrumb() -> Result<()> {
    let items = items_from_results(&[result("lvl1")])?;

    assert_eq!(
        items[0].subtitle(),
        Some("Docs & Guides > Background & Color")
    );
    Ok(())
}

#[test]
fn item_preserves_url_fields() -> Result<()> {
    let items = items_from_results(&[result("lvl1")])?;
    let item = &items[0];

    assert_eq!(
        (item.arg(), item.quick_look_url(), item.valid()),
        (
            Some("https://tailwindcss.com/docs/background-color"),
            Some("https://tailwindcss.com/docs/background-color"),
            true,
        )
    );
    Ok(())
}

#[test]
fn item_rejects_missing_selected_hierarchy_level() {
    let error = items_from_results(&[result("lvl2")])
        .expect_err("missing hierarchy level must be rejected");

    assert_eq!(
        error.to_string(),
        "Algolia result background-color is missing hierarchy level 2"
    );
}

#[test]
fn item_rejects_invalid_result_type() {
    let error =
        items_from_results(&[result("heading")]).expect_err("invalid result type must be rejected");

    assert_eq!(error.to_string(), "invalid Algolia result type: heading");
}

#[test]
fn google_fallback_encodes_query_and_is_selectable() -> Result<()> {
    let item = google_fallback_item("background color")?;

    assert_eq!(
        (item.arg(), item.valid()),
        (
            Some("https://www.google.com/search?q=Tailwind+CSS+background+color"),
            true,
        )
    );
    Ok(())
}

#[test]
fn placeholder_is_not_selectable() {
    assert!(!placeholder_item().valid());
}
