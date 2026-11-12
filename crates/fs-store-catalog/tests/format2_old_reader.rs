//! What the OLD reader (this code, v0.0.12) does with catalogs in format 2 (fs-catalog v0.0.1, `docs/format.md`).
//! The tests ASSERT the observed behaviour; none of them repairs anything. See BERICHT.md for the table.

use fs_store_catalog::{parse_catalog, Catalog};
use fs_store_core::{Description, Kind};

const EXAMPLE: &str = include_str!("fixtures/catalog.example.xml");

const SIG: &str = r#"<value key="fs:signature">https://x/p.asc</value>"#;

/// One component with a Linux x86_64 artifact; `extra` goes into `<custom>`, `inside` into `<component>`.
fn component(id: &str, kind: &str, extra: &str, inside: &str) -> String {
    format!(
        r#"<component type="desktop-application"><id>{id}</id><name>{id}</name><summary>s {id}</summary>{inside}
<releases><release version="1.0.0" date="2026-11-12"><artifacts><artifact type="binary" platform="x86_64-linux-gnu">
<location>https://x/{id}.tar.gz</location><checksum type="sha256">aa</checksum><size type="download">10</size>
</artifact></artifacts></release></releases>
<custom><value key="fs:kind">{kind}</value>{SIG}{extra}</custom></component>"#
    )
}

fn catalog(root_attrs: &str, components: &[String]) -> String {
    format!(
        r#"<?xml version="1.0"?><components version="1.0" {root_attrs}>{}</components>"#,
        components.join("\n")
    )
}

fn parse(xml: &str) -> Catalog {
    parse_catalog(xml).expect("the whole catalog must not fail")
}

fn names(c: &Catalog) -> Vec<&str> {
    c.entries.iter().map(|e| e.name.as_str()).collect()
}

fn skipped(c: &Catalog) -> Vec<(&str, &str)> {
    c.skipped
        .iter()
        .map(|s| (s.id.as_str(), s.reason.as_str()))
        .collect()
}

#[test]
fn example_catalog_whole_is_read_server_and_icon_theme_are_left_out() {
    let c = parse(EXAMPLE);
    assert_eq!(names(&c), ["example-notes"]);
    assert_eq!(
        skipped(&c),
        [
            ("example-server", "unknown fs:kind `server`"),
            ("example-icons", "unknown fs:kind `icon-theme`"),
        ]
    );
    // Header fields are read as numbers; `fs-schema` is not looked at at all.
    assert_eq!((c.serial, c.expires), (Some(0), Some(0)));
}

#[test]
fn example_program_stays_usable_with_every_format_2_field() {
    let c = parse(EXAMPLE);
    let e = &c.entries[0];
    assert_eq!(e.kind, Kind::Program);
    assert_eq!(e.version.to_string(), "0.0.2");
    assert_eq!(e.location, "https://x/example-notes/example-notes-0.0.2-linux-x86_64.tar.gz");
    assert!(e.signature_url.ends_with(".tar.gz.asc"));
    assert!(e.icon.is_some());
    assert_eq!(e.details.size, Some(123_456));
    // Several `xml:lang` descriptions: the FIRST one wins (the format writes `en` first), no language choice.
    assert!(e.description.starts_with("Shows every field of the catalog format."));
    assert!(!e.description.contains("Zeigt"));
    // The wiki block id is carried, the description text is not used.
    assert_eq!(
        e.details.description,
        Description::WikiBlock("0192f3a0-7b1c-7a3e-8c4d-5e6f7a8b9c0d".to_owned())
    );
    // Picture with hash and licence is shown; caption is the first (en) one.
    assert_eq!(e.details.screenshots.len(), 1);
    assert_eq!(e.details.screenshots[0].caption, "The main window");
    // History: both releases listed, newest first.
    let versions: Vec<_> = e.details.releases.iter().map(|r| r.version.as_str()).collect();
    assert_eq!(versions, ["0.0.2", "0.0.1"]);
    // Capabilities are read; `fs:feature.*`, `<provides><mediatype>`, `fs:maker` are simply not looked at.
    println!("unreadable: {:?}", e.details.unreadable);
}

#[test]
fn kind_server_alone_is_skipped() {
    let c = parse(&catalog("fs-schema=\"2\"", &[component("srv", "server", "", "")]));
    assert!(c.entries.is_empty());
    assert_eq!(skipped(&c), [("srv", "unknown fs:kind `server`")]);
}

#[test]
fn kind_icon_theme_alone_is_skipped() {
    let c = parse(&catalog("fs-schema=\"2\"", &[component("ico", "icon-theme", "", "")]));
    assert!(c.entries.is_empty());
    assert_eq!(skipped(&c), [("ico", "unknown fs:kind `icon-theme`")]);
}

#[test]
fn unknown_kind_does_not_hide_the_known_entries_around_it() {
    let xml = catalog(
        "fs-schema=\"2\"",
        &[
            component("first", "program", "", ""),
            component("srv", "server", "", ""),
            component("ico", "icon-theme", "", ""),
            component("last", "program", "", ""),
        ],
    );
    let c = parse(&xml);
    assert_eq!(names(&c), ["first", "last"]);
    assert_eq!(c.skipped.len(), 2);
}

#[test]
fn min_store_99_on_a_known_kind_is_ignored_the_old_store_offers_it() {
    // NOT protected: an old store does not read `fs:min-store` (the format doc says so too).
    let extra = r#"<value key="fs:min-store">99.0.0</value>"#;
    let c = parse(&catalog("fs-schema=\"2\"", &[component("new", "program", extra, "")]));
    assert_eq!(names(&c), ["new"]);
    assert!(c.skipped.is_empty());
}

#[test]
fn min_store_99_on_an_unknown_kind_is_skipped_by_the_kind() {
    let extra = r#"<value key="fs:min-store">99.0.0</value>"#;
    let c = parse(&catalog("fs-schema=\"2\"", &[component("new", "server", extra, "")]));
    assert!(c.entries.is_empty());
    assert_eq!(skipped(&c), [("new", "unknown fs:kind `server`")]);
}

#[test]
fn lang_variants_first_description_wins_even_when_it_is_german() {
    let inside = r#"<description xml:lang="de"><p>Deutsch.</p></description><description xml:lang="en"><p>English.</p></description>"#;
    let c = parse(&catalog("", &[component("l", "program", "", inside)]));
    assert_eq!(names(&c), ["l"]);
    assert_eq!(c.entries[0].description, "Deutsch.");
}

#[test]
fn lang_variants_on_name_summary_and_captions_do_not_break_reading() {
    let inside = r#"<name xml:lang="de">Namen</name><summary xml:lang="de">Kurz</summary>"#;
    let c = parse(&catalog("", &[component("l", "program", "", inside)]));
    assert_eq!(names(&c), ["l"]);
    // Observed: the first `<name>`/`<summary>` is the one the component helper wrote; extra variants are ignored.
    assert_eq!(c.entries[0].summary, "s l");
}

#[test]
fn unknown_attributes_elements_and_custom_keys_are_ignored() {
    let inside = r#"<future-element a="1"><x/></future-element>"#;
    let extra = r#"<value key="fs:from-the-future">1</value><value key="fs:maker">M</value>"#;
    let xml = catalog(
        r#"fs-schema="3" fs-future="yes" origin="freesynergy""#,
        &[component("fut", "program", extra, inside).replace(
            r#"<id>"#,
            r#"<id fs-new="x">"#,
        )],
    );
    let c = parse(&xml);
    assert_eq!(names(&c), ["fut"]);
    assert!(c.skipped.is_empty());
}

#[test]
fn description_wiki_with_block_id_is_carried_as_wiki_block() {
    let extra = r#"<value key="fs:description-wiki">blk-1</value><value key="fs:description-wiki-source-rev">3</value>"#;
    let c = parse(&catalog("", &[component("w", "program", extra, "")]));
    assert_eq!(c.entries[0].details.description, Description::WikiBlock("blk-1".to_owned()));
}

#[test]
fn provides_requires_with_unknown_or_odd_names_do_not_fail_the_entry() {
    let extra = r#"<value key="fs:provides">identity.oidc 1.0, ???, no.such.name>=2</value><value key="fs:requires">people.persons 1.2</value>"#;
    let c = parse(&catalog("", &[component("p", "program", extra, "")]));
    assert_eq!(names(&c), ["p"]);
    println!("provides: {:?} unreadable: {:?}", c.entries[0].details.provides, c.entries[0].details.unreadable);
}

#[test]
fn history_releases_without_files_do_not_become_the_offered_version() {
    let inside = r#"<releases><release version="0.9.0" date="2026-01-01"><description xml:lang="en"><p>Old.</p></description></release></releases>"#;
    let c = parse(&catalog("", &[component("h", "program", "", inside)]));
    // The component helper has its own `<releases>`; the first `<releases>` here has only the history entry,
    // the offered release is still found because the search walks all descendants.
    assert_eq!(names(&c), ["h"]);
    assert_eq!(c.entries[0].version.to_string(), "1.0.0");
}

#[test]
fn history_only_entry_without_any_artifact_is_skipped_not_an_error() {
    let xml = r#"<components version="1.0" fs-schema="2"><component type="desktop-application"><id>none</id>
<releases><release version="0.9.0"><description><p>Old.</p></description></release></releases>
<custom><value key="fs:signature">https://x/p.asc</value></custom></component></components>"#;
    let c = parse(xml);
    assert!(c.entries.is_empty());
    assert_eq!(skipped(&c), [("none", "no release with an artifact for this machine")]);
}

#[test]
fn image_without_hash_or_licence_is_withheld_not_an_error() {
    let inside = r#"<screenshots><screenshot><image type="source" fs-license="proprietary">https://x/a.png</image></screenshot></screenshots>"#;
    let c = parse(&catalog("", &[component("i", "program", "", inside)]));
    assert_eq!(names(&c), ["i"]);
    assert!(c.entries[0].details.screenshots.is_empty());
    assert_eq!(c.entries[0].details.images_withheld.len(), 1);
}

#[test]
fn schema_2_in_the_header_is_not_checked_so_no_error_for_the_whole_catalog() {
    for attrs in [r#"fs-schema="2""#, r#"fs-schema="99""#, r#"fs-schema="x""#, r#"schema="2""#] {
        let c = parse(&catalog(attrs, &[component("a", "program", "", "")]));
        assert_eq!(names(&c), ["a"], "{attrs}");
    }
}

#[test]
fn zero_expiry_from_the_example_header_parses_as_some_zero() {
    // `fs-expires="0"` is Some(0). The freshness check (fs-store-sources `Serials::check`: `now > e` => Expired)
    // then calls such a catalog expired. Only the example file carries 0; see BERICHT.md.
    let c = parse(&catalog(r#"fs-serial="0" fs-expires="0""#, &[]));
    assert_eq!((c.serial, c.expires), (Some(0), Some(0)));
}
