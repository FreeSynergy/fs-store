#![allow(clippy::assert_is_empty)] // test data
//! "Cancel": where it may stop and what is left. Every test has a counter-proof: after the stop the machine is as it was before
//! (the folder of the old version, the `current` link, no half unpacked tree, no leftover temporary folder), and without the stop
//! the same data installs.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

use fs_store_core::*;
use fs_store_engine::service::{Fetch, InstallFailure, Installer};
use sha2::{Digest, Sha256};

struct Map(HashMap<String, Vec<u8>>, Mutex<Vec<Progress>>);
impl Fetch for Map {
    fn get(&self, url: &str) -> Result<Vec<u8>, String> {
        self.0.get(url).cloned().ok_or_else(|| format!("404 {url}"))
    }
    /// Reads in four pieces, looks at the token between them and reports each (as a real download does).
    fn get_controlled(
        &self,
        url: &str,
        cancel: &CancelToken,
        progress: &(dyn Fn(Progress) + Sync),
    ) -> Result<Vec<u8>, String> {
        let all = self.get(url)?;
        let total = all.len() as u64;
        for i in 1..=4u64 {
            if cancel.is_cancelled() {
                return Err(CANCELLED.to_owned());
            }
            let p = Progress {
                done: total * i / 4,
                total: Some(total),
            };
            self.1.lock().unwrap().push(p);
            progress(p);
        }
        Ok(all)
    }
}
struct Fake;
impl SignatureVerifier for Fake {
    fn verify(&self, _: &[u8], sig: Option<&[u8]>) -> SignatureOutcome {
        match sig {
            Some(b"good") => SignatureOutcome::Valid {
                fingerprint: TRUSTED_FINGERPRINT.into(),
            },
            _ => SignatureOutcome::Invalid,
        }
    }
}
struct Idle;
impl RunningCheck for Idle {
    fn is_running(&self, _: &str) -> bool {
        false
    }
}

/// A tar.gz of `n` files.
fn tarball(n: usize, content: &str) -> Vec<u8> {
    let mut raw = Vec::new();
    let enc = flate2::write::GzEncoder::new(&mut raw, flate2::Compression::fast());
    let mut b = tar::Builder::new(enc);
    for i in 0..n {
        let mut h = tar::Header::new_gnu();
        h.set_size(content.len() as u64);
        h.set_mode(0o755);
        h.set_cksum();
        b.append_data(&mut h, format!("bin/prog{i}"), content.as_bytes())
            .unwrap();
    }
    b.into_inner().unwrap().finish().unwrap();
    raw
}

fn setup(version: &str, files: usize) -> (Map, Vec<u8>, Vec<u8>) {
    let tar = tarball(files, version);
    let sha = format!("{:x}", Sha256::digest(&tar));
    let catalog = format!(
        r#"<components version="1.0"><component type="desktop-application"><id>prog</id>
<releases><release version="{version}"><artifacts><artifact type="binary"><location>https://x/prog.tar.gz</location>
<checksum type="sha256">{sha}</checksum></artifact></artifacts></release></releases>
<custom><value key="fs:signature">https://x/prog.asc</value></custom></component></components>"#
    )
    .into_bytes();
    let fetch = Map(
        HashMap::from([
            ("https://x/prog.tar.gz".into(), tar),
            ("https://x/prog.asc".into(), b"good".to_vec()),
        ]),
        Mutex::new(Vec::new()),
    );
    (fetch, catalog, b"good".to_vec())
}

fn installer<'a>(root: &Path, fetch: &'a Map) -> Installer<'a> {
    Installer {
        root: root.to_owned(),
        verifier: &Fake,
        fetch,
        running: &Idle,
        prefer: Format::TarGz,
        policy: SignaturePolicy::default(),
    }
}

fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .map(|d| {
            d.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// Installs `version` normally, as the state to come back to.
fn install(root: &Path, version: &str) {
    let (fetch, catalog, sig) = setup(version, 3);
    let i = installer(root, &fetch);
    i.install(&catalog, &sig, "prog", None).unwrap();
}

#[test]
fn cancel_in_the_middle_of_a_download_changes_nothing_and_without_it_the_same_data_installs() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    let (fetch, catalog, sig) = setup("1.0.0", 3);
    let i = installer(&root, &fetch);
    let plan = i.plan(&catalog, &sig, "prog", None).unwrap();
    // the token sets itself at the third look: two pieces are read, then the stop
    let stopped = i.download_controlled(&plan, &CancelToken::after_looks(3), &|_| {});
    assert!(
        matches!(stopped, Err(InstallFailure::Cancelled)),
        "{stopped:?}"
    );
    assert_eq!(names(&root), Vec::<String>::new(), "nothing on the machine");
    // counter-proof: the same data, not cancelled, installs; progress was reported in pieces
    let ok = i
        .download_controlled(&plan, &CancelToken::new(), &|_| {})
        .unwrap();
    let prepared = i.check(&plan, ok).unwrap();
    assert_eq!(
        i.commit_all(vec![prepared])[0].1.as_ref().unwrap(),
        &Version::new(1, 0, 0)
    );
    let seen = fetch.1.lock().unwrap();
    assert!(
        seen.iter().any(|p| p.share() == Some(0.5)) && seen.iter().any(|p| p.share() == Some(1.0))
    );
}

#[test]
fn cancel_after_the_checks_and_before_the_install_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    let (fetch, catalog, sig) = setup("1.0.0", 3);
    let i = installer(&root, &fetch);
    let plan = i.plan(&catalog, &sig, "prog", None).unwrap();
    let cancel = CancelToken::new();
    let prepared = i
        .check(
            &plan,
            i.download_controlled(&plan, &cancel, &|_| {}).unwrap(),
        )
        .unwrap();
    cancel.cancel();
    let done = i.commit_all(vec![prepared]);
    assert!(matches!(done[0].1, Err(InstallFailure::Cancelled)));
    assert_eq!(names(&root), Vec::<String>::new());
}

#[test]
fn cancel_in_the_middle_of_unpacking_leaves_the_old_version_current_and_no_half_tree() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    install(&root, "1.0.0");
    let before = (
        names(&root.join("packages/prog")),
        fs::read_link(root.join("current/prog")).unwrap(),
    );

    let (fetch, catalog, sig) = setup("1.1.0", 40);
    let i = installer(&root, &fetch);
    let plan = i
        .plan(&catalog, &sig, "prog", Some(Version::new(1, 0, 0)))
        .unwrap();
    let cancel = CancelToken::new();
    let prepared = i
        .check(
            &plan,
            i.download_controlled(&plan, &cancel, &|_| {}).unwrap(),
        )
        .unwrap();
    // the token sets itself at the tenth look at it: the unpacking has written part of the tree by then
    let prepared = prepared_with(prepared, CancelToken::after_looks(10));
    let done = i.commit_all(vec![prepared]);
    assert!(
        matches!(done[0].1, Err(InstallFailure::Cancelled)),
        "{:?}",
        done[0].1
    );

    let after = (
        names(&root.join("packages/prog")),
        fs::read_link(root.join("current/prog")).unwrap(),
    );
    assert_eq!(
        after, before,
        "the old version is current, the new one has no folder, no temporary folder is left"
    );
    assert!(
        root.join("current/prog/bin/prog0").is_file(),
        "the old files are in place"
    );
    assert_eq!(
        fs::read_to_string(root.join("current/prog/bin/prog0")).unwrap(),
        "1.0.0"
    );

    // counter-proof: the same unpacking without the stop installs the new version completely
    let (fetch, catalog, sig) = setup("1.1.0", 40);
    let i = installer(&root, &fetch);
    i.install(&catalog, &sig, "prog", Some(Version::new(1, 0, 0)))
        .unwrap();
    assert_eq!(names(&root.join("packages/prog")), ["1.0.0", "1.1.0"]);
    assert_eq!(
        fs::read_dir(root.join("packages/prog/1.1.0/bin"))
            .unwrap()
            .count(),
        40
    );
}

/// Gives a prepared package another token (the test's way to cancel at a chosen look).
fn prepared_with(p: fs_store_engine::Prepared, token: CancelToken) -> fs_store_engine::Prepared {
    p.with_cancel(token)
}
