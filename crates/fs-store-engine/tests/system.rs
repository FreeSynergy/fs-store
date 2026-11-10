//! The store offers the package of the system it runs on; what it cannot install says `Unsupported`.
#![allow(
    clippy::format_push_string,
    clippy::case_sensitive_file_extension_comparisons
)] // test data
use std::collections::HashMap;

use fs_store_core::*;
use fs_store_engine::service::{Fetch, InstallFailure, Installer};
use fs_store_engine::{System, format_for, system_from_os_release};
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

/// One program whose release has a tar.gz, rpm, deb and exe file (no `platform`: fits every machine).
fn catalog(name: &str, blobs: &[(&str, &str, &[u8])]) -> String {
    let mut arts = String::new();
    for (format, file, bytes) in blobs {
        arts += &format!(
            r#"<artifact type="binary" fs-format="{format}" fs-signature="https://x/{file}.asc"><location>https://x/{file}</location><checksum type="sha256">{:x}</checksum></artifact>"#,
            Sha256::digest(bytes)
        );
    }
    format!(
        r#"<components version="1.0"><component type="desktop-application"><id>{name}</id>
<releases><release version="1.0.0"><artifacts><artifact type="binary"><location>https://x/{name}.tar.gz</location>
<checksum type="sha256">00</checksum></artifact>{arts}</artifacts></release></releases>
<custom><value key="fs:signature">https://x/{name}.asc</value></custom></component></components>"#
    )
}

fn installer<'a>(root: &std::path::Path, fetch: &'a Map, prefer: Format) -> Installer<'a> {
    Installer {
        root: root.join("root"),
        verifier: &Fake,
        fetch,
        running: &Idle,
        prefer,
        policy: SignaturePolicy::default(),
    }
}

const BLOBS: [(&str, &str, &[u8]); 3] = [
    ("rpm", "app.rpm", b"RPM"),
    ("deb", "app.deb", b"DEB"),
    ("exe", "app-setup.exe", b"EXE"),
];

fn served() -> Map {
    let mut map = HashMap::new();
    for (_, f, b) in BLOBS {
        map.insert(format!("https://x/{f}"), b.to_vec());
        map.insert(format!("https://x/{f}.asc"), b"good".to_vec());
    }
    Map(map)
}

#[test]
fn os_release_names_the_package_system() {
    let ubuntu = "NAME=\"Ubuntu\"\nID=ubuntu\nID_LIKE=debian\n";
    let debian = "PRETTY_NAME=\"Debian GNU/Linux 13 (trixie)\"\nID=debian\n";
    let mint = "ID=linuxmint\nID_LIKE=\"ubuntu debian\"\n";
    let fedora = "ID=fedora\nVERSION_ID=45\n";
    let rocky = "ID=\"rocky\"\nID_LIKE=\"rhel centos fedora\"\n";
    let suse = "ID=\"opensuse-leap\"\nID_LIKE=\"suse opensuse\"\n";
    let arch = "ID=arch\n";
    assert_eq!(system_from_os_release(ubuntu), System::Deb);
    assert_eq!(system_from_os_release(debian), System::Deb);
    assert_eq!(system_from_os_release(mint), System::Deb);
    assert_eq!(system_from_os_release(fedora), System::Rpm);
    assert_eq!(system_from_os_release(rocky), System::Rpm);
    assert_eq!(system_from_os_release(suse), System::Other); // zypper, not dnf
    assert_eq!(system_from_os_release(arch), System::Other);
    assert_eq!(system_from_os_release(""), System::Other);
}

#[test]
fn the_format_follows_the_system_and_how_the_store_came() {
    assert_eq!(format_for(System::Rpm, true), Format::Rpm);
    assert_eq!(format_for(System::Deb, true), Format::Deb);
    assert_eq!(format_for(System::Rpm, false), Format::TarGz);
    assert_eq!(format_for(System::Deb, false), Format::TarGz);
    assert_eq!(format_for(System::Other, true), Format::TarGz);
    assert_eq!(format_for(System::Windows, false), Format::Exe);
}

#[test]
fn each_system_is_offered_its_own_files() {
    let tmp = tempfile::tempdir().unwrap();
    let cat = catalog("app", &BLOBS);
    let fetch = served();
    for (prefer, location) in [
        (Format::Rpm, "https://x/app.rpm"),
        (Format::Deb, "https://x/app.deb"),
        (Format::TarGz, "https://x/app.tar.gz"),
    ] {
        let plan = installer(tmp.path(), &fetch, prefer)
            .plan(cat.as_bytes(), b"good", "app", None)
            .unwrap();
        assert_eq!(plan.format(), prefer, "{location}");
        assert_eq!(plan.locations(), [location]);
    }
}

#[test]
fn a_deb_store_falls_back_to_the_tar_gz_when_the_release_has_no_deb() {
    let tmp = tempfile::tempdir().unwrap();
    let cat = catalog("app", &BLOBS[..1]); // rpm only
    let plan = installer(tmp.path(), &served(), Format::Deb)
        .plan(cat.as_bytes(), b"good", "app", None)
        .unwrap();
    assert_eq!(plan.format(), Format::TarGz);
}

#[test]
fn a_windows_store_is_told_unsupported_not_given_something_else() {
    let tmp = tempfile::tempdir().unwrap();
    let cat = catalog("app", &BLOBS);
    let r =
        installer(tmp.path(), &served(), Format::Exe).plan(cat.as_bytes(), b"good", "app", None);
    match r {
        Err(e @ InstallFailure::Unsupported(Format::Exe)) => {
            assert!(e.to_string().starts_with("Unsupported"), "{e}");
        }
        other => panic!(
            "expected Unsupported(Exe), got {:?}",
            other.map(|p| p.format())
        ),
    }
}

#[test]
fn all_deb_of_a_batch_go_to_one_pkexec_apt_call() {
    let tmp = tempfile::tempdir().unwrap();
    let cat = catalog("app", &BLOBS);
    let fetch = served();
    let i = installer(tmp.path(), &fetch, Format::Deb);
    let plan = i.plan(cat.as_bytes(), b"good", "app", None).unwrap();
    let ready: Vec<_> = i
        .prepare_all(vec![plan], &|_, _| {})
        .into_iter()
        .map(|(_, r)| r.unwrap())
        .collect();
    let calls = Calls::default();
    let seen = std::sync::Arc::clone(&calls.0);
    let done = i.commit_deb_with(ready, &|_, _| {}, calls);
    assert!(done.iter().all(|(_, r)| r.is_ok()));
    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    assert_eq!(
        seen[0][..5],
        ["pkexec", "apt-get", "install", "-y", "--no-remove"]
    );
    assert!(seen[0][5].ends_with(".deb"));
}

#[test]
fn a_main_file_that_is_a_deb_is_not_given_to_a_store_of_another_kind() {
    let tmp = tempfile::tempdir().unwrap();
    let cat =
        catalog("app", &[]).replace("<custom>", r#"<custom><value key="fs:format">deb</value>"#);
    let fetch = served();
    let other =
        installer(tmp.path(), &fetch, Format::TarGz).plan(cat.as_bytes(), b"good", "app", None);
    assert!(matches!(
        other,
        Err(InstallFailure::Unsupported(Format::Deb))
    ));
    let deb = installer(tmp.path(), &fetch, Format::Deb).plan(cat.as_bytes(), b"good", "app", None);
    assert_eq!(deb.unwrap().format(), Format::Deb);
}
