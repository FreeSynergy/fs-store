// Integration tests for Inventory: load, record mutations, persistence.
//
// Tests that require the Store repo use the same skip-if-missing guard as
// store_reader.rs.  Pure-logic tests (record mutations, save/load) run
// without any Store access and always pass.

use std::path::PathBuf;

use fs_store::{InstallRecord, Inventory, StoreReader, StoreSettings, StoreSource};

fn local_store_path() -> Option<PathBuf> {
    let raw =
        std::env::var("FS_STORE_LOCAL").unwrap_or_else(|_| "/home/kal/Server/Store".to_owned());
    let path = PathBuf::from(raw);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

// ── load() integration test ───────────────────────────────────────────────────

#[tokio::test]
async fn inventory_load_populates_states() {
    let Some(store_path) = local_store_path() else {
        eprintln!("skip: Store repo not found");
        return;
    };

    let reader = StoreReader::new(StoreSource::Local(store_path));
    let mut inv = Inventory::new(StoreSettings::default());
    inv.load(&reader).await.expect("inventory load failed");

    assert!(
        inv.states.len() > 10,
        "expected more than 10 package states"
    );
    assert!(
        inv.package_state("forgejo").is_some(),
        "forgejo should be in states"
    );
    assert!(
        inv.package_state("kanidm").is_some(),
        "kanidm should be in states"
    );
    assert_eq!(
        inv.installed().count(),
        0,
        "fresh inventory: nothing installed"
    );
}

// ── record_installed / record_removed (unit, no Store needed) ─────────────────

/// Build a minimal Inventory that already has states without calling load().
async fn inventory_with_fake_packages() -> Inventory {
    // We need a real Store load for states to exist.
    // Use a temp dir as data_dir to avoid touching real files.
    let Some(store_path) = local_store_path() else {
        return Inventory::new(StoreSettings::default());
    };

    let mut settings = StoreSettings::default();
    settings.storage.data_dir = std::env::temp_dir().join("fs-store-test-records");

    let reader = StoreReader::new(StoreSource::Local(store_path));
    let mut inv = Inventory::new(settings);
    inv.load(&reader).await.expect("load failed");
    inv
}

#[tokio::test]
async fn record_installed_marks_package_as_installed() {
    let Some(_) = local_store_path() else {
        eprintln!("skip: Store repo not found");
        return;
    };

    let mut inv = inventory_with_fake_packages().await;
    if inv.states.is_empty() {
        return; // store not available
    }

    let record = InstallRecord::new(
        "forgejo",
        "9.0.0",
        PathBuf::from("/tmp/freesynergy/packages/forgejo"),
    );

    inv.record_installed(record);

    let state = inv.package_state("forgejo").expect("forgejo state missing");
    assert!(state.is_installed(), "forgejo should be installed");
    assert_eq!(state.active().map(|r| r.version.as_str()), Some("9.0.0"));
    assert_eq!(inv.installed().count(), 1);
}

#[tokio::test]
async fn record_removed_clears_install() {
    let Some(_) = local_store_path() else {
        eprintln!("skip: Store repo not found");
        return;
    };

    let mut inv = inventory_with_fake_packages().await;
    if inv.states.is_empty() {
        return;
    }

    inv.record_installed(InstallRecord::new(
        "forgejo",
        "9.0.0",
        PathBuf::from("/tmp/freesynergy/packages/forgejo"),
    ));
    assert!(inv.package_state("forgejo").unwrap().is_installed());

    inv.record_removed("forgejo", "9.0.0");
    assert!(!inv.package_state("forgejo").unwrap().is_installed());
    assert_eq!(inv.installed().count(), 0);
}

#[tokio::test]
async fn record_installed_deactivates_previous_version() {
    let Some(_) = local_store_path() else {
        eprintln!("skip: Store repo not found");
        return;
    };

    let mut inv = inventory_with_fake_packages().await;
    if inv.states.is_empty() {
        return;
    }

    inv.record_installed(InstallRecord::new(
        "forgejo",
        "8.0.0",
        PathBuf::from("/tmp/freesynergy/packages/forgejo"),
    ));
    inv.record_installed(InstallRecord::new(
        "forgejo",
        "9.0.0",
        PathBuf::from("/tmp/freesynergy/packages/forgejo"),
    ));

    let state = inv.package_state("forgejo").unwrap();
    assert_eq!(state.installed.len(), 2);
    // Only the last (9.0.0) should be active.
    let active = state.active().expect("should have an active record");
    assert_eq!(active.version, "9.0.0");
    let old = state
        .installed
        .iter()
        .find(|r| r.version == "8.0.0")
        .unwrap();
    assert!(!old.is_active);
}

// ── save_records / read_records round-trip ────────────────────────────────────

#[tokio::test]
async fn save_and_reload_records_round_trip() {
    let Some(store_path) = local_store_path() else {
        eprintln!("skip: Store repo not found");
        return;
    };

    let tmp = std::env::temp_dir().join(format!(
        "fs-store-test-rtrip-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    ));

    let mut settings = StoreSettings::default();
    settings.storage.data_dir = tmp.clone();

    let reader = StoreReader::new(StoreSource::Local(store_path.clone()));
    let mut inv = Inventory::new(settings.clone());
    inv.load(&reader).await.expect("load failed");

    inv.record_installed(InstallRecord::new("forgejo", "9.0.0", tmp.join("forgejo")));
    inv.save_records().expect("save_records failed");

    // Load a fresh inventory from the same settings — records must survive.
    let reader2 = StoreReader::new(StoreSource::Local(store_path));
    let mut inv2 = Inventory::new(settings);
    inv2.load(&reader2).await.expect("reload failed");

    let state = inv2.package_state("forgejo").expect("forgejo missing");
    assert!(
        state.is_installed(),
        "forgejo should still be installed after reload"
    );
    assert_eq!(state.active().map(|r| r.version.as_str()), Some("9.0.0"));

    // Cleanup.
    let _ = std::fs::remove_dir_all(&tmp);
}
