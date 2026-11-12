//! Tests of the reader for catalog format 2: kinds this store does not install are shown locked and never fail anything, texts in several
//! languages are chosen by the language of the program, and the new fields are read.

use fs_store_catalog::{
    COMING_SOON, NOT_SUPPORTED, UNREADABLE_MIN_STORE, parse_catalog, parse_catalog_in,
};
use fs_store_core::{Description, Kind};

fn artifact(extra: &str) -> String {
    format!(
        r#"<artifacts><artifact type="binary"><location>https://example.org/x.tar.gz</location><checksum type="sha256">x</checksum>{extra}</artifact></artifacts>"#
    )
}

fn component(id: &str, kind: &str, body: &str) -> String {
    format!(
        r#"<component type="desktop-application"><id>{id}</id><name>{id}</name>{body}
<custom><value key="fs:signature">https://example.org/{id}.asc</value><value key="fs:kind">{kind}</value></custom></component>"#
    )
}

fn good(id: &str) -> String {
    component(
        id,
        "program",
        &format!(
            r#"<releases><release version="1.0.0">{}</release></releases>"#,
            artifact("")
        ),
    )
}

fn catalog(parts: &[String]) -> String {
    format!(
        r#"<components version="1.0">{}</components>"#,
        parts.concat()
    )
}

#[test]
fn a_server_entry_is_locked_and_the_packages_next_to_it_still_load() {
    // the server has no release file at all: still no error, and what can be read is read
    let server = component(
        "net.freesynergy.srv",
        "server",
        r#"<summary>A background service</summary><description><p>Runs in the background.</p></description>
<releases><release version="2.0.0"/></releases>"#,
    );
    let c = parse_catalog(&catalog(&[
        good("net.freesynergy.a"),
        server,
        good("net.freesynergy.b"),
    ]))
    .unwrap();
    assert_eq!(c.skipped.len(), 0);
    assert_eq!(c.entries.len(), 3);
    let srv = &c.entries[1];
    assert_eq!(srv.kind, Kind::Server);
    assert_eq!(srv.locked.as_deref(), Some(COMING_SOON));
    assert!(!srv.installable());
    assert_eq!(srv.summary, "A background service");
    assert_eq!(srv.description, "Runs in the background.");
    assert_eq!(srv.version.to_string(), "2.0.0");
    // counter-proof: the packages before and after are installable and unchanged
    for e in [&c.entries[0], &c.entries[2]] {
        assert!(e.installable(), "{} must stay installable", e.name.as_str());
        assert_eq!(e.kind, Kind::Program);
    }
}

#[test]
fn an_icon_theme_and_a_kind_nobody_knows_are_locked_too() {
    let c = parse_catalog(&catalog(&[
        component("net.freesynergy.icons", "icon-theme", ""),
        component("net.freesynergy.odd", "hologram", ""),
        good("net.freesynergy.a"),
    ]))
    .unwrap();
    assert_eq!(c.skipped.len(), 0);
    assert_eq!(c.entries[0].kind, Kind::IconTheme);
    assert_eq!(c.entries[0].locked.as_deref(), Some(COMING_SOON));
    assert_eq!(c.entries[1].kind, Kind::Unsupported);
    assert_eq!(c.entries[1].kind_text, "hologram");
    assert_eq!(c.entries[1].locked.as_deref(), Some(NOT_SUPPORTED));
    assert!(c.entries[2].installable());
}

#[test]
fn an_entry_that_wants_a_newer_store_is_locked_and_one_that_wants_an_older_is_not() {
    let with = |min: &str| {
        component(
            "net.freesynergy.n",
            "program",
            &format!(
                r#"<releases><release version="1.0.0">{}</release></releases>"#,
                artifact("")
            ),
        )
        .replace(
            "</custom>",
            &format!(r#"<value key="fs:min-store">{min}</value></custom>"#),
        )
    };
    let newer = parse_catalog(&catalog(&[with("99.0.0")])).unwrap();
    assert_eq!(newer.entries[0].locked.as_deref(), Some(NOT_SUPPORTED));
    let older = parse_catalog(&catalog(&[with("0.0.1")])).unwrap();
    assert!(older.entries[0].installable());
}

#[test]
fn an_entry_with_a_known_kind_but_no_release_file_is_still_reported_as_before() {
    // a locked kind is tolerant; a program without a file is a real problem and is listed as skipped
    let broken = component("net.freesynergy.broken", "program", "");
    let c = parse_catalog(&catalog(&[broken, good("net.freesynergy.a")])).unwrap();
    assert_eq!(c.entries.len(), 1);
    assert_eq!(c.skipped.len(), 1);
}

const LANGS: &str = r#"<components version="1.0"><component type="desktop-application"><id>net.freesynergy.l</id>
<summary>English summary</summary><summary xml:lang="de">Deutsche Kurzfassung</summary>
<description><p>English text.</p></description><description xml:lang="de"><p>Deutscher Text.</p></description><description xml:lang="fr"><p>Texte.</p></description>
<screenshots><screenshot><image type="source" fs-sha256="h" fs-license="MIT" fs-source="https://maker.example/shot.png">https://example.org/1.png</image>
<caption>Main window</caption><caption xml:lang="de">Hauptfenster</caption></screenshot></screenshots>
<releases><release version="1.0.0" date="2026-11-01"><description><p>First.</p></description><description xml:lang="de"><p>Erstes.</p></description>
<artifacts><artifact type="binary"><location>https://example.org/l.tar.gz</location><checksum type="sha256">l</checksum>
<size type="download">1000</size><size type="installed">4096</size></artifact></artifacts></release></releases>
<custom><value key="fs:signature">https://example.org/l.asc</value></custom></component></components>"#;

fn prefs(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn texts_come_in_the_language_of_the_program_then_english_then_the_first() {
    let read = |p: &[&str]| {
        parse_catalog_in(LANGS, &prefs(p))
            .unwrap()
            .entries
            .remove(0)
    };
    let de = read(&["de", "en"]);
    assert_eq!(de.description, "Deutscher Text.");
    assert_eq!(de.summary, "Deutsche Kurzfassung");
    assert_eq!(de.details.changes, "Erstes.");
    assert_eq!(de.details.releases[0].text, "Erstes.");
    assert_eq!(de.details.screenshots[0].caption, "Hauptfenster");
    // counter-proof: English asked, English given; and plain parse_catalog is English
    let en = read(&["en"]);
    assert_eq!(en.description, "English text.");
    assert_eq!(en.details.screenshots[0].caption, "Main window");
    assert_eq!(
        parse_catalog(LANGS).unwrap().entries[0].summary,
        "English summary"
    );
    // a language the catalog does not have falls back to English, and so does the second choice
    assert_eq!(read(&["ja", "en"]).description, "English text.");
    assert_eq!(read(&["fr", "en"]).description, "Texte.");
}

#[test]
fn when_neither_the_language_nor_english_is_there_the_first_variant_is_taken() {
    let xml = r#"<components><component type="desktop-application"><id>net.freesynergy.f</id>
<description xml:lang="fr"><p>Texte.</p></description><description xml:lang="it"><p>Testo.</p></description>
<releases><release version="1.0.0"><artifacts><artifact type="binary"><location>https://example.org/f.tar.gz</location>
<checksum type="sha256">f</checksum></artifact></artifacts></release></releases>
<custom><value key="fs:signature">https://example.org/f.asc</value></custom></component></components>"#;
    let e = parse_catalog_in(xml, &prefs(&["de", "en"]))
        .unwrap()
        .entries
        .remove(0);
    assert_eq!(e.description, "Texte.");
}

#[test]
fn the_new_fields_are_read() {
    let e = parse_catalog_in(LANGS, &prefs(&["en"]))
        .unwrap()
        .entries
        .remove(0);
    assert_eq!(e.details.size, Some(1000));
    assert_eq!(e.details.installed_size, Some(4096));
    assert_eq!(
        e.details.screenshots[0].source,
        "https://maker.example/shot.png"
    );

    let xml = r#"<components><component type="desktop-application"><id>net.freesynergy.m</id>
<provides><mediatype>text/calendar</mediatype><mediatype>text/vcard</mediatype><mediatype>text/calendar</mediatype></provides>
<releases><release version="1.0.0"><artifacts><artifact type="binary"><location>https://example.org/m.tar.gz</location>
<checksum type="sha256">m</checksum></artifact></artifacts></release></releases>
<custom><value key="fs:signature">https://example.org/m.asc</value>
<value key="fs:maker">Example Maker</value>
<value key="fs:description-wiki">0190a1f2-7c3b-7d10-9a55-2f4e8b6c1d30</value>
<value key="fs:description-wiki-source-rev">14</value>
<value key="fs:description-wiki-lang">de</value>
<value key="fs:features">contacts, calendar</value>
<value key="fs:feature.contacts.name">Contacts</value>
<value key="fs:feature.contacts.provides">people.persons 1.0</value>
<value key="fs:feature.contacts.requires">identity.accounts 1.1</value>
<value key="fs:feature.calendar.recommends">mail.send 1.0, no.such.name 1.0</value></custom></component></components>"#;
    let d = parse_catalog(xml).unwrap().entries.remove(0).details;
    assert_eq!(d.origin.maker, "Example Maker");
    assert_eq!(d.mediatypes, ["text/calendar", "text/vcard"]);
    let Description::WikiBlock(w) = &d.description else {
        panic!("a wiki block is expected");
    };
    assert_eq!(w.block, "0190a1f2-7c3b-7d10-9a55-2f4e8b6c1d30");
    assert_eq!(w.source_rev, Some(14));
    assert_eq!(w.lang, "de");
    assert_eq!(d.features.len(), 2);
    assert_eq!(d.features[0].key, "contacts");
    assert_eq!(d.features[0].name, "Contacts");
    assert_eq!(d.features[0].provides[0].to_string(), "people.persons 1.0");
    assert_eq!(
        d.features[0].requires[0].to_string(),
        "identity.accounts 1.1"
    );
    assert_eq!(d.features[1].recommends.len(), 2);
    // counter-proof: an entry without these fields has none of them
    let plain = parse_catalog_in(LANGS, &prefs(&["en"]))
        .unwrap()
        .entries
        .remove(0)
        .details;
    assert_eq!(plain.features.len(), 0);
    assert_eq!(plain.mediatypes.len(), 0);
    assert_eq!(plain.origin.maker, "");
    assert!(matches!(plain.description, Description::Text(_)));
}

#[test]
fn an_unreadable_min_store_locks_the_entry_and_an_empty_one_does_not() {
    let with = |min: &str| {
        component(
            "net.freesynergy.u",
            "program",
            &format!(
                r#"<releases><release version="1.0.0">{}</release></releases>"#,
                artifact("")
            ),
        )
        .replace(
            "</custom>",
            &format!(r#"<value key="fs:min-store">{min}</value></custom>"#),
        )
    };
    for bad in ["0.1", "soon", "1.x.0"] {
        let c = parse_catalog(&catalog(&[with(bad)])).unwrap();
        assert_eq!(c.skipped.len(), 0, "{bad}");
        assert_eq!(
            c.entries[0].locked.as_deref(),
            Some(UNREADABLE_MIN_STORE),
            "{bad}"
        );
        assert!(!c.entries[0].installable());
    }
    // counter-proof: a readable one that fits, and a value that is empty (= not said)
    assert!(parse_catalog(&catalog(&[with("0.0.1")])).unwrap().entries[0].installable());
    assert!(parse_catalog(&catalog(&[with("")])).unwrap().entries[0].installable());
}

#[test]
fn a_locked_entry_without_a_release_file_makes_no_statement_about_the_signature() {
    let server = component("net.freesynergy.srv", "server", "");
    let c = parse_catalog(&catalog(&[server, good("net.freesynergy.a")])).unwrap();
    assert_eq!(c.entries[0].details.trust, None);
    // counter-proof: a signed entry with a release file keeps its level; a locked one WITH a release file keeps it too
    assert!(c.entries[1].details.trust.is_some());
    let wants_newer = good("net.freesynergy.n").replace(
        "</custom>",
        r#"<value key="fs:min-store">99.0.0</value></custom>"#,
    );
    let n = parse_catalog(&catalog(&[wants_newer])).unwrap();
    assert!(n.entries[0].locked.is_some() && n.entries[0].details.trust.is_some());
}
