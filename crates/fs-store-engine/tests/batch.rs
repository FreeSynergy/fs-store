#![allow(
    clippy::assert_is_empty,
    clippy::many_single_char_names,
    clippy::format_push_string,
    clippy::case_sensitive_file_extension_comparisons
)] // test data
mod common;

use std::collections::HashMap;
use std::fs;

use fs_store_core::*;
use fs_store_engine::service::{Fetch, InstallFailure, Installer};
use sha2::{Digest, Sha256};

struct Map(HashMap<String, Vec<u8>>);
impl Fetch for Map {
    fn get(&self, url: &str) -> Result<Vec<u8>, String> {
        self.0.get(url).cloned().ok_or_else(|| format!("404 {url}"))
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

fn tarball(content: &str) -> Vec<u8> {
    let mut raw = Vec::new();
    let enc = flate2::write::GzEncoder::new(&mut raw, flate2::Compression::fast());
    let mut b = tar::Builder::new(enc);
    let mut h = tar::Header::new_gnu();
    h.set_size(content.len() as u64);
    h.set_mode(0o755);
    h.set_cksum();
    b.append_data(&mut h, "bin/prog", content.as_bytes())
        .unwrap();
    b.into_inner().unwrap().finish().unwrap();
    raw
}

fn component(name: &str, sha: &str) -> String {
    format!(
        r#"<component type="desktop-application"><id>{name}</id>
<releases><release version="1.0.0"><artifacts><artifact type="binary"><location>https://x/{name}.tar.gz</location>
<checksum type="sha256">{sha}</checksum></artifact></artifacts></release></releases>
<custom><value key="fs:signature">https://x/{name}.asc</value></custom></component>"#
    )
}

#[test]
fn a_failed_download_does_not_stop_the_others() {
    let tmp = tempfile::tempdir().unwrap();
    let (a, b, c) = (tarball("a"), tarball("b"), tarball("c"));
    let sha = |x: &[u8]| format!("{:x}", Sha256::digest(x));
    let catalog = format!(
        r#"<components version="1.0">{}{}{}</components>"#,
        component("alpha", &sha(&a)),
        component("beta", &sha(&b)),
        component("gamma", &sha(&c)),
    );
    // `beta` cannot be downloaded
    let fetch = Map(HashMap::from([
        ("https://x/alpha.tar.gz".into(), a),
        ("https://x/alpha.asc".into(), b"good".to_vec()),
        ("https://x/gamma.tar.gz".into(), c),
        ("https://x/gamma.asc".into(), b"good".to_vec()),
    ]));
    let i = Installer {
        root: tmp.path().join("root"),
        verifier: &Fake,
        fetch: &fetch,
        running: &Idle,
        prefer: Format::TarGz,
        policy: SignaturePolicy::default(),
    };
    let plans = ["alpha", "beta", "gamma"]
        .map(|n| i.plan(catalog.as_bytes(), b"good", n, None).unwrap())
        .into();
    let prepared = i.prepare_all(plans, &|_, _| {});
    let mut failed = Vec::new();
    let mut ready = Vec::new();
    for (name, r) in prepared {
        match r {
            Ok(p) => ready.push(p),
            Err(e) => failed.push((name, e)),
        }
    }
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].0, "beta");
    assert!(matches!(failed[0].1, InstallFailure::Fetch(_)));
    let done = i.commit_all(ready);
    assert_eq!(done.len(), 2);
    assert!(done.iter().all(|(_, r)| *r == Ok(Version::new(1, 0, 0))));
    let root = tmp.path().join("root");
    assert_eq!(
        fs::read_to_string(root.join("current/alpha/bin/prog")).unwrap(),
        "a"
    );
    assert_eq!(
        fs::read_to_string(root.join("current/gamma/bin/prog")).unwrap(),
        "c"
    );
    assert!(!root.join("current/beta").exists());
}

#[test]
fn nothing_is_installed_before_every_download_is_done() {
    // prepare_all never touches the root: only commit does
    let tmp = tempfile::tempdir().unwrap();
    let a = tarball("a");
    let catalog = format!(
        r#"<components version="1.0">{}</components>"#,
        component("alpha", &format!("{:x}", Sha256::digest(&a)))
    );
    let fetch = Map(HashMap::from([
        ("https://x/alpha.tar.gz".into(), a),
        ("https://x/alpha.asc".into(), b"good".to_vec()),
    ]));
    let i = Installer {
        root: tmp.path().join("root"),
        verifier: &Fake,
        fetch: &fetch,
        running: &Idle,
        prefer: Format::TarGz,
        policy: SignaturePolicy::default(),
    };
    let plan = i.plan(catalog.as_bytes(), b"good", "alpha", None).unwrap();
    let prepared = i.prepare_all(vec![plan], &|_, _| {});
    assert!(prepared[0].1.is_ok());
    assert!(!tmp.path().join("root").exists());
}

#[derive(Default)]
struct Calls(std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>);
impl fs_store_engine::Commander for Calls {
    fn run(&mut self, program: &str, args: &[&str]) -> Result<(), String> {
        let mut line = vec![program.to_owned()];
        line.extend(args.iter().map(|a| (*a).to_owned()));
        self.0.lock().unwrap().push(line);
        Ok(())
    }
}

fn rpm_component(name: &str, files: &[(&str, &[u8])]) -> String {
    let mut arts = String::new();
    for (n, (file, bytes)) in files.iter().enumerate() {
        arts += &format!(
            r#"<artifact type="binary" fs-format="rpm" fs-signature="https://x/{file}.asc"><location>https://x/{file}.rpm</location><checksum type="sha256">{:x}</checksum></artifact>"#,
            Sha256::digest(bytes)
        );
        let _ = n;
    }
    format!(
        r#"<component type="desktop-application"><id>{name}</id>
<releases><release version="1.0.0"><artifacts><artifact type="binary"><location>https://x/{name}.tar.gz</location>
<checksum type="sha256">00</checksum></artifact>{arts}</artifacts></release></releases>
<custom><value key="fs:signature">https://x/{name}.asc</value></custom></component>"#
    )
}

#[cfg(unix)]
#[test]
fn all_rpm_of_a_batch_go_to_one_pkexec_call() {
    let tmp = tempfile::tempdir().unwrap();
    let blobs: Vec<(&str, &[u8])> = vec![("one", b"1"), ("two", b"2"), ("three", b"3")];
    let catalog = format!(
        r#"<components version="1.0">{}{}</components>"#,
        rpm_component("alpha", &blobs[..2]),
        rpm_component("beta", &blobs[2..]),
    );
    let mut map = HashMap::new();
    for (f, b) in &blobs {
        map.insert(format!("https://x/{f}.rpm"), b.to_vec());
        map.insert(format!("https://x/{f}.asc"), b"good".to_vec());
    }
    let fetch = Map(map);
    let i = Installer {
        root: tmp.path().join("root"),
        verifier: &Fake,
        fetch: &fetch,
        running: &Idle,
        prefer: Format::Rpm,
        policy: SignaturePolicy::default(),
    };
    let plans = vec![
        i.plan(catalog.as_bytes(), b"good", "alpha", None).unwrap(),
        i.plan(catalog.as_bytes(), b"good", "beta", None).unwrap(),
    ];
    let ready: Vec<_> = i
        .prepare_all(plans, &|_, _| {})
        .into_iter()
        .map(|(_, r)| r.unwrap())
        .collect();
    let calls = Calls::default();
    let seen = std::sync::Arc::clone(&calls.0);
    let done = i.commit_rpm_with(ready, &|_, _| {}, calls);
    assert_eq!(done.len(), 2);
    assert!(done.iter().all(|(_, r)| r.is_ok()));
    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 1, "one password for the whole batch");
    assert_eq!(seen[0][0], "pkexec");
    assert_eq!(seen[0].iter().filter(|a| a.ends_with(".rpm")).count(), 3);
}

#[test]
fn a_background_server_does_not_block_the_update() {
    let tmp = tempfile::tempdir().unwrap();
    let program = common::server_copy(tmp.path(), "fs-blockedname");
    let mut server = std::process::Command::new(program)
        .arg("serve")
        .spawn()
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(300));
    let tgz = tarball("a");
    let catalog = format!(
        r#"<components version="1.0">{}</components>"#,
        component("fs-blockedname", &format!("{:x}", Sha256::digest(&tgz)))
    );
    let fetch = Map(HashMap::from([
        ("https://x/fs-blockedname.tar.gz".into(), tgz),
        ("https://x/fs-blockedname.asc".into(), b"good".to_vec()),
    ]));
    let i = Installer {
        root: tmp.path().join("root"),
        verifier: &Fake,
        fetch: &fetch,
        running: &Idle,
        prefer: Format::TarGz,
        policy: SignaturePolicy::default(),
    };
    let plan = i
        .plan(catalog.as_bytes(), b"good", "fs-blockedname", None)
        .unwrap();
    let ready: Vec<_> = i
        .prepare_all(vec![plan], &|_, _| {})
        .into_iter()
        .map(|(_, r)| r.unwrap())
        .collect();
    #[cfg(unix)]
    {
        let reaper = std::thread::spawn(move || server.wait().unwrap());
        let done = i.commit_all(ready);
        assert_eq!(done[0].1, Ok(Version::new(1, 0, 0)));
        reaper.join().unwrap(); // the server was ended by the store
        assert!(fs_store_engine::find_servers("fs-blockedname", 0).is_empty());
    }
    // Windows cannot ask a program without a window to end: the update says so and the program keeps running.
    #[cfg(windows)]
    {
        let done = i.commit_all(ready);
        assert!(done[0].1.is_err());
        assert!(server.try_wait().unwrap().is_none());
        server.kill().unwrap();
        server.wait().unwrap();
    }
}
