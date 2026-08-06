use anyhow::{Result, anyhow};
use serde::Deserialize;

/// A Tailwind CSS documentation result returned by Algolia.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct SearchResult {
    /// Algolia object identifier.
    #[serde(rename = "objectID")]
    pub object_id: String,
    /// Algolia hierarchy level tag, such as `lvl2`.
    #[serde(rename = "type")]
    pub result_type: String,
    /// Documentation URL opened by Alfred.
    pub url: String,
    /// Documentation hierarchy used for titles and breadcrumbs.
    pub hierarchy: SearchResultHierarchy,
    /// Optional searchable page content.
    pub content: Option<String>,
}

impl SearchResult {
    /// Returns the hierarchy level encoded by the result type.
    pub fn hierarchy_level(&self) -> Result<usize> {
        let level = self
            .result_type
            .strip_prefix("lvl")
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|level| *level <= 6)
            .ok_or_else(|| anyhow!("invalid Algolia result type: {}", self.result_type))?;

        Ok(level)
    }
}

/// Ordered hierarchy values returned for a documentation result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct SearchResultHierarchy {
    /// Root hierarchy value.
    pub lvl0: String,
    /// Level-one hierarchy value.
    pub lvl1: Option<String>,
    /// Level-two hierarchy value.
    pub lvl2: Option<String>,
    /// Level-three hierarchy value.
    pub lvl3: Option<String>,
    /// Level-four hierarchy value.
    pub lvl4: Option<String>,
    /// Level-five hierarchy value.
    pub lvl5: Option<String>,
    /// Level-six hierarchy value.
    pub lvl6: Option<String>,
}

impl SearchResultHierarchy {
    /// Returns the hierarchy value at a level from zero through six.
    pub fn level(&self, level: usize) -> Option<&str> {
        match level {
            0 => Some(&self.lvl0),
            1 => self.lvl1.as_deref(),
            2 => self.lvl2.as_deref(),
            3 => self.lvl3.as_deref(),
            4 => self.lvl4.as_deref(),
            5 => self.lvl5.as_deref(),
            6 => self.lvl6.as_deref(),
            _ => None,
        }
    }

    /// Iterates over all populated hierarchy values in display order.
    pub fn values(&self) -> impl Iterator<Item = &str> {
        [
            Some(self.lvl0.as_str()),
            self.lvl1.as_deref(),
            self.lvl2.as_deref(),
            self.lvl3.as_deref(),
            self.lvl4.as_deref(),
            self.lvl5.as_deref(),
            self.lvl6.as_deref(),
        ]
        .into_iter()
        .flatten()
    }
}

/// Minimal subset of an Algolia single-index search response.
#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    /// Search results in provider-defined ranking order.
    pub hits: Vec<SearchResult>,
}
