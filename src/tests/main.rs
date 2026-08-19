use std::cell::Cell;

use alfred_workflow_rs::FileCache;

use super::*;

fn settings(version: &str) -> WorkflowSettings {
    WorkflowSettings {
        tailwind_version: version.to_owned(),
        use_alfred_cache: false,
        use_file_cache: false,
        cache_ttl: Some(86_400),
        file_cache_max_entries: Some(1_280),
    }
}

#[test]
fn plist_defaults_map_to_runtime_settings() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let info_path = directory.path().join("info.plist");
    std::fs::write(&info_path, include_str!("../../info.plist"))?;

    let actual = read_workflow_settings(
        &Workflow::new(),
        info_path,
        directory.path().join("missing-prefs.plist"),
    )?;

    assert_eq!(
        actual,
        WorkflowSettings {
            tailwind_version: "v4".to_owned(),
            use_alfred_cache: true,
            use_file_cache: false,
            cache_ttl: Some(86_400),
            file_cache_max_entries: Some(1_280),
        }
    );
    Ok(())
}

#[test]
fn automatic_cache_wins_when_both_modes_are_enabled() {
    let mut workflow = Workflow::new();
    let mut settings = settings("v4");
    settings.use_alfred_cache = true;
    settings.use_file_cache = true;

    configure_cache(&mut workflow, "background", &settings);

    assert_eq!(
        (workflow.use_automatic_cache(), workflow.cache_key()),
        (true, None)
    );
}

#[test]
fn file_cache_key_includes_normalized_query_and_version() {
    assert_eq!(
        file_cache_key("background color", "v4"),
        "background color_v4"
    );
}

#[test]
fn empty_query_shows_placeholder_without_searching() -> Result<()> {
    let search_calls = Cell::new(0);
    let cli = Cli::default();
    let mut workflow = Workflow::new();

    populate_workflow_with(&mut workflow, &cli, &settings("v4"), |_, _| {
        search_calls.set(search_calls.get() + 1);
        Ok(Vec::new())
    })?;

    assert_eq!(
        (search_calls.get(), workflow.get_items()?.items()[0].title()),
        (0, "Search the Tailwind CSS docs...")
    );
    Ok(())
}

#[test]
fn empty_query_does_not_enter_file_cache() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut settings = settings("v4");
    settings.use_file_cache = true;
    let search_calls = Cell::new(0);
    let cli = Cli::default();
    let mut first = Workflow::with_file_cache(FileCache::with_path(directory.path()));

    populate_workflow_with(&mut first, &cli, &settings, |_, _| {
        search_calls.set(search_calls.get() + 1);
        Ok(Vec::new())
    })?;

    let mut second = Workflow::with_file_cache(FileCache::with_path(directory.path()));
    populate_workflow_with(&mut second, &cli, &settings, |_, _| {
        search_calls.set(search_calls.get() + 1);
        Ok(Vec::new())
    })?;

    assert_eq!(
        (
            search_calls.get(),
            first.cache_key(),
            first.get_items()?.len(),
            second.cache_key(),
            second.get_items()?.len(),
        ),
        (0, None, 1, None, 1)
    );
    Ok(())
}

#[test]
fn file_cache_hit_bypasses_algolia() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut cached = Workflow::with_file_cache(FileCache::with_path(directory.path()));
    cached.set_cache_key(Some("background_v4"));
    let cached_item = google_fallback_item("background")?;
    cached.add_item(cached_item.clone())?;

    let mut workflow = Workflow::with_file_cache(FileCache::with_path(directory.path()));
    let mut settings = settings("v4");
    settings.use_file_cache = true;
    let search_calls = Cell::new(0);
    let cli = Cli {
        query: "background".to_owned(),
        ..Cli::default()
    };

    populate_workflow_with(&mut workflow, &cli, &settings, |_, _| {
        search_calls.set(search_calls.get() + 1);
        Ok(Vec::new())
    })?;

    assert_eq!(search_calls.get(), 0);
    assert_eq!(workflow.get_items()?.items(), &[cached_item]);
    Ok(())
}

#[test]
fn file_cache_entries_are_version_specific() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut cached = Workflow::with_file_cache(FileCache::with_path(directory.path()));
    cached.set_cache_key(Some("background_v3"));
    cached.add_item(Item::new("v3 result"))?;

    let mut workflow = Workflow::with_file_cache(FileCache::with_path(directory.path()));
    let mut settings = settings("v4");
    settings.use_file_cache = true;
    let search_calls = Cell::new(0);
    let cli = Cli {
        query: "background".to_owned(),
        ..Cli::default()
    };

    populate_workflow_with(&mut workflow, &cli, &settings, |_, _| {
        search_calls.set(search_calls.get() + 1);
        Ok(Vec::new())
    })?;

    assert_eq!(search_calls.get(), 1);
    Ok(())
}

#[test]
fn update_item_is_rendered_without_entering_file_cache() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut workflow = Workflow::with_file_cache(FileCache::with_path(directory.path()));
    workflow.set_cache_key(Some("background_v4"));
    workflow.add_item(Item::new("search result"))?;
    let options = update_render_options_with(&Cli::default(), || Ok(true));

    let rendered: serde_json::Value =
        serde_json::from_str(&workflow.to_json_string_with(options)?)?;
    let cached = workflow.get_items()?;

    assert_eq!(
        (
            rendered["items"].as_array().map(Vec::len),
            cached.len(),
            cached.items()[0].title()
        ),
        (Some(2), 1, "search result")
    );
    Ok(())
}
