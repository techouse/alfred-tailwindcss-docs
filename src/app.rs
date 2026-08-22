use alfred_workflow_rs::{Icon, Item, ItemText};
use anyhow::{Result, anyhow};
use htmlize::unescape;
use url::Url;

use crate::models::SearchResult;

/// Builds the placeholder shown before the user enters a search query.
pub fn placeholder_item() -> Item {
    Item::new("Search the Tailwind CSS docs...").set_icon(Icon::new("icon.png"))
}

/// Converts ranked Tailwind search results into Alfred items in provider order.
pub fn items_from_results(results: &[SearchResult]) -> Result<Vec<Item>> {
    results.iter().map(item_from_result).collect()
}

/// Builds the Google fallback shown when Algolia returns no hits.
pub fn google_fallback_item(query: &str) -> Result<Item> {
    let url = Url::parse_with_params(
        "https://www.google.com/search",
        [("q", format!("Tailwind CSS {query}"))],
    )?;

    Ok(Item::builder("No matching answers found")
        .subtitle("Shall I try and search Google?")
        .arg(url.as_str())
        .text(ItemText::new(url.as_str()))
        .quick_look_url(url.as_str())
        .icon(Icon::new("google.png"))
        .valid(true)
        .build()?)
}

fn item_from_result(result: &SearchResult) -> Result<Item> {
    let level = result.hierarchy_level()?;
    let title = result.hierarchy.level(level).ok_or_else(|| {
        anyhow!(
            "Algolia result {} is missing hierarchy level {level}",
            result.object_id
        )
    })?;
    let mut builder = Item::builder(title)
        .uid(&result.object_id)
        .arg(&result.url)
        .text(ItemText::new(&result.url).with_large_type(title))
        .quick_look_url(&result.url)
        .icon(Icon::new("icon.png"))
        .valid(true);

    if level > 0 {
        let breadcrumb = result.hierarchy.values().collect::<Vec<_>>().join(" > ");
        builder = builder.subtitle(decode_html_text(&breadcrumb));
    }

    Ok(builder.build()?)
}

fn decode_html_text(text: &str) -> String {
    let mut decoded = String::with_capacity(text.len());
    let mut segment_start = 0;
    let mut cursor = 0;

    while let Some(relative_start) = text[cursor..].find("&#") {
        let start = cursor + relative_start;
        let Some((end, character)) = legacy_numeric_reference(text, start) else {
            cursor = start + 2;
            continue;
        };

        decoded.push_str(unescape(&text[segment_start..start]).as_ref());
        if let Some(character) = character {
            decoded.push(character);
        } else {
            decoded.push_str(&text[start..end]);
        }
        cursor = end;
        segment_start = end;
    }

    decoded.push_str(unescape(&text[segment_start..]).as_ref());
    decoded
}

fn legacy_numeric_reference(text: &str, start: usize) -> Option<(usize, Option<char>)> {
    let bytes = text.as_bytes();
    let mut cursor = start + 2;
    let (radix, uppercase_x) = match bytes.get(cursor) {
        Some(b'x') => {
            cursor += 1;
            (16, false)
        }
        Some(b'X') => {
            cursor += 1;
            (16, true)
        }
        _ => (10, false),
    };
    let digits_start = cursor;
    while let Some(byte) = bytes.get(cursor) {
        let is_digit = if radix == 16 {
            byte.is_ascii_hexdigit()
        } else {
            byte.is_ascii_digit()
        };
        if !is_digit {
            break;
        }
        cursor += 1;
    }

    if cursor == digits_start {
        return None;
    }

    let terminated = bytes.get(cursor) == Some(&b';');
    let end = if terminated { cursor + 1 } else { cursor };
    if !terminated || uppercase_x {
        return Some((end, None));
    }

    let digits = &text[digits_start..cursor];
    let character = u32::from_str_radix(digits, radix)
        .ok()
        .and_then(char::from_u32);
    Some((end, character))
}

#[cfg(test)]
#[path = "tests/app.rs"]
mod tests;
