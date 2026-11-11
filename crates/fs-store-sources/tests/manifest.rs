use fs_store_sources::{Basis, FileRecord, Manifest, ManifestStore};

fn sample(name: &str) -> Manifest {
    Manifest {
        schema: 1,
        name: name.into(),
        version: "1.0.0".into(),
        source: "https://example.org/cat".into(),
        signer: "ABCD".into(),
        basis: Basis::Installed,
        recorded_at: 7,
        files: vec![FileRecord {
            path: "bin/x".into(),
            hash: "sha256:00".into(),
            size: 1,
            mode: 0o755,
        }],
    }
}

#[test]
fn a_list_is_saved_read_listed_and_forgotten() {
    let tmp = tempfile::tempdir().unwrap();
    let store = ManifestStore::new(tmp.path());
    assert!(store.load("x").unwrap().is_none());
    store.save(&sample("x")).unwrap();
    store.save(&sample("a")).unwrap();
    assert_eq!(store.load("x").unwrap().unwrap(), sample("x"));
    assert_eq!(store.names().unwrap(), ["a", "x"]);
    store.forget("x").unwrap();
    store.forget("x").unwrap();
    assert_eq!(store.names().unwrap(), ["a"]);
}

#[test]
fn a_name_that_is_not_a_plain_file_name_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let store = ManifestStore::new(tmp.path());
    assert!(store.save(&sample("../evil")).is_err());
    assert!(store.load(".hidden").is_err());
    assert!(store.forget("a/b").is_err());
}

#[test]
fn the_text_form_is_stable_and_marks_the_basis() {
    let tmp = tempfile::tempdir().unwrap();
    let store = ManifestStore::new(tmp.path());
    let mut m = sample("x");
    m.basis = Basis::Retroactive;
    store.save(&m).unwrap();
    let text = std::fs::read_to_string(tmp.path().join("manifests/x.toml")).unwrap();
    assert!(text.contains("basis = \"retroactive\""), "{text}");
    assert!(text.contains("hash = \"sha256:00\""));
}
