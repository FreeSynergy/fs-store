//! Pattern: none (tests of the wire writer).
//! The new fields of `Package` on the wire (`store.capnp` @7 to @18): what the catalog says arrives, what it does not say stays at
//! the default of the contract (so a reader shows "unknown"), and a package known only from the register carries none of it.

use fs_interfaces::store_capnp::{Trust, package};
use fs_store_core::{
    Capability, Description, Details, Format, Kind, Origin, ReleaseNote, Screenshot, TrustLevel,
};

use crate::view::PackageInfo;
use crate::wire::write_packages;

fn info(name: &str, details: Details) -> PackageInfo {
    PackageInfo {
        name: name.into(),
        kind: Kind::Component,
        version: "1.5.0".into(),
        format: Format::TarGz,
        source: "freesynergy".into(),
        facets: Vec::new(),
        summary: "Identity management".into(),
        details,
        icon_sha256: "ab12".into(),
    }
}

fn caps(text: &str) -> Vec<Capability> {
    Capability::parse_list(text).0
}

fn written(list: &[PackageInfo]) -> capnp::message::Builder<capnp::message::HeapAllocator> {
    let mut message = capnp::message::Builder::new_default();
    let out = message.initn_root::<capnp::struct_list::Builder<package::Owned>>(
        u32::try_from(list.len()).unwrap(),
    );
    write_packages(list, out);
    message
}

#[test]
#[allow(clippy::too_many_lines)] // one long list of expectations on one written package
fn what_the_catalog_says_arrives_and_every_name_of_a_capability_is_kept() {
    let rich = Details {
        size: Some(13_631_488),
        released: "2026-11-09".into(),
        trust: Some(TrustLevel::MakerAndMirror),
        origin: Origin {
            source: "https://example.invalid/catalog".into(),
            upstream: "https://github.com/kanidm/kanidm".into(),
            upstream_signature: "https://example.invalid/k.asc".into(),
            source_code: "https://example.invalid/k-src.tar.gz".into(),
        },
        provides: caps("identity.oidc 1.0, mail.send 1.2"),
        requires: caps("people.persons >= 1.2"),
        recommends: caps("groups"),
        releases: vec![ReleaseNote {
            version: "1.5.0".into(),
            date: "2026-11-09".into(),
            text: "Fixes the sign-in loop.".into(),
        }],
        screenshots: vec![Screenshot {
            url: "https://example.invalid/1.png".into(),
            sha256: "00".into(),
            caption: "Main".into(),
            license: "CC-BY-4.0".into(),
        }],
        description: Description::WikiBlock("wiki:store/kanidm".into()),
        ..Details::default()
    };
    let message = written(&[info("kanidm", rich)]);
    let list = message
        .get_root_as_reader::<capnp::struct_list::Reader<package::Owned>>()
        .unwrap();
    let p = list.get(0);
    assert_eq!(p.get_size(), 13_631_488);
    assert_eq!(
        p.get_released().unwrap().get_unix_millis(),
        1_794_182_400_000
    );
    assert_eq!(p.get_trust().unwrap(), Trust::MakerAndMirror);
    assert_eq!(
        p.get_summary().unwrap().to_str().unwrap(),
        "Identity management"
    );
    assert_eq!(p.get_icon_sha256().unwrap().to_str().unwrap(), "ab12");
    let o = p.get_origin().unwrap();
    assert_eq!(
        o.get_source().unwrap().to_str().unwrap(),
        "https://github.com/kanidm/kanidm"
    );
    assert_eq!(
        o.get_source_code().unwrap().to_str().unwrap(),
        "https://example.invalid/k-src.tar.gz"
    );
    let provides = p.get_provides().unwrap();
    assert_eq!(provides.len(), 2);
    // `mail.send` is not in the list of fs-interfaces: it is written all the same (the reader shows it as it is)
    assert!(!fs_interfaces::capabilities::is_known("mail.send"));
    assert_eq!(
        provides.get(1).get_name().unwrap().to_str().unwrap(),
        "mail.send"
    );
    assert_eq!(
        provides.get(1).get_version().unwrap().to_str().unwrap(),
        "1.2"
    );
    assert!(fs_interfaces::capabilities::is_known("people.persons"));
    assert_eq!(
        p.get_requires()
            .unwrap()
            .get(0)
            .get_version()
            .unwrap()
            .to_str()
            .unwrap(),
        "1.2"
    );
    assert_eq!(
        p.get_recommends()
            .unwrap()
            .get(0)
            .get_version()
            .unwrap()
            .to_str()
            .unwrap(),
        ""
    );
    let r = p.get_releases().unwrap().get(0);
    assert_eq!(
        r.get_changes().unwrap().to_str().unwrap(),
        "Fixes the sign-in loop."
    );
    assert_eq!(
        r.get_released().unwrap().get_unix_millis(),
        1_794_182_400_000
    );
    assert_eq!(
        p.get_pictures()
            .unwrap()
            .get(0)
            .get_license()
            .unwrap()
            .to_str()
            .unwrap(),
        "CC-BY-4.0"
    );
    match p.get_description().which().unwrap() {
        package::description::WikiBlock(id) => {
            assert_eq!(id.unwrap().to_str().unwrap(), "wiki:store/kanidm");
        }
        package::description::Text(_) => panic!("the description is a wiki block"),
    }
}

#[test]
fn what_the_catalog_does_not_say_stays_at_the_default_of_the_contract() {
    let message = written(&[info("fs-mystery", Details::default())]);
    let list = message
        .get_root_as_reader::<capnp::struct_list::Reader<package::Owned>>()
        .unwrap();
    let p = list.get(0);
    assert_eq!(p.get_size(), 0, "0 = not said");
    assert_eq!(p.get_released().unwrap().get_unix_millis(), 0);
    assert_eq!(p.get_trust().unwrap(), Trust::Unknown);
    assert_eq!(p.get_releases().unwrap().len(), 0);
    assert_eq!(p.get_provides().unwrap().len(), 0);
    assert_eq!(p.get_pictures().unwrap().len(), 0);
    assert_eq!(
        p.get_origin()
            .unwrap()
            .get_source()
            .unwrap()
            .to_str()
            .unwrap(),
        ""
    );
    // the older fields are as before
    assert_eq!(p.get_name().unwrap().to_str().unwrap(), "fs-mystery");
    assert_eq!(p.get_version().unwrap().to_str().unwrap(), "1.5.0");
}
