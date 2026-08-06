#![forbid(unsafe_code)]

//! Tailwind CSS documentation search and Alfred Script Filter support.

/// Conversion from Tailwind search results to Alfred items.
pub mod app;
/// Search-result models returned by Algolia.
pub mod models;
/// External services used by the workflow.
pub mod services;
