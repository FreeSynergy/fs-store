// Integration tests for StoreReader against the local Store catalog.
//
// These tests require the Store repository to be checked out at the path
// in FS_STORE_LOCAL (default: ~/Server/Store).  They are skipped when the
// directory is not present so CI without the full repo still passes.

use fs_store::{StoreReader, StoreSource};

fn local_store_path() -> Option<std::path::PathBuf> {
    // Allow override via env var; fall back to the default dev path.
    let raw = std::env::var("FS_STORE_LOCAL")
        .unwrap_or_else(|_| "/home/kal/Server/Store".to_owned());
    let path = std::path::PathBuf::from(raw);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

#[tokio::test]
async fn load_all_returns_expected_counts() {
    let Some(store_path) = local_store_path() else {
        eprintln!("skip: Store repo not found");
        return;
    };

    let reader = StoreReader::new(StoreSource::Local(store_path));
    let map = reader.load_all().await.expect("load_all failed");

    // Apps: node, kanidm, stalwart, tuwunel, zentinel, zentinel-control-plane, mistral, browser
    assert_eq!(map.apps.len(), 8, "expected 8 app packages");

    // Containers: forgejo, postgres, outline, cryptpad, dragonfly, vikunja, pretix, umap, openobserver, otel-collector
    assert_eq!(map.containers.len(), 10, "expected 10 container packages");

    // Widgets: clock, system-info, messages, my-tasks, quick-notes, weather
    assert_eq!(map.widgets.len(), 6, "expected 6 widget packages");

    // Themes: midnight-blue, nordic-dark, cloud-white
    assert_eq!(map.themes.len(), 3, "expected 3 theme packages");

    // Bundles: zentinel
    assert_eq!(map.bundles.len(), 1, "expected 1 bundle package");

    // Icons: freesynergy-default
    assert_eq!(map.icons.len(), 1, "expected 1 icon set");

    // Repos: freesynergy-community
    assert_eq!(map.repos.len(), 1, "expected 1 repo package");

    println!("total packages: {}", map.total_count());
}

#[tokio::test]
async fn kanidm_fields_are_correct() {
    let Some(store_path) = local_store_path() else {
        eprintln!("skip: Store repo not found");
        return;
    };

    let reader = StoreReader::new(StoreSource::Local(store_path));
    let map = reader.load_all().await.expect("load_all failed");

    let kanidm = map
        .apps
        .iter()
        .find(|p| p.id() == "kanidm")
        .expect("kanidm not found");

    assert_eq!(kanidm.name(), "Kanidm");
    assert!(kanidm.summary().contains("identity") || kanidm.summary().contains("Identity"));
    assert_eq!(kanidm.latest_version(), Some("1.4.2"));
    assert!(
        kanidm.icon_path().is_some(),
        "kanidm should have an icon_path"
    );
    assert!(kanidm.tags().contains(&"oidc".to_owned()));
}

#[tokio::test]
async fn load_namespace_languages_is_non_empty() {
    let Some(store_path) = local_store_path() else {
        eprintln!("skip: Store repo not found");
        return;
    };

    let reader = StoreReader::new(StoreSource::Local(store_path));
    let langs = reader
        .load_namespace("packages/i18n")
        .await
        .expect("load languages failed");

    assert!(!langs.is_empty(), "expected at least one language pack");

    let de = langs.iter().find(|p| p.id() == "de").expect("de not found");
    assert_eq!(de.name(), "Deutsch");
}

#[tokio::test]
async fn load_namespace_empty_tasks_does_not_fail() {
    let Some(store_path) = local_store_path() else {
        eprintln!("skip: Store repo not found");
        return;
    };

    let reader = StoreReader::new(StoreSource::Local(store_path));
    // tasks namespace exists but has no [[packages]] entries
    let tasks = reader
        .load_namespace("packages/tasks")
        .await
        .expect("load tasks failed");

    assert_eq!(tasks.len(), 0, "tasks namespace should be empty");
}
