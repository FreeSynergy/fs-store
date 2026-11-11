//! Tests of the package details read from the catalog (ADR 0068).

use fs_store_catalog::parse_catalog;

const XML: &str = r#"<components version="1.0"><component type="desktop-application"><id>net.freesynergy.plain</id>
<releases><release version="0.8.0"><artifacts><artifact type="binary"><location>https://example.org/p.tar.gz</location>
<checksum type="sha256">p</checksum></artifact></artifacts></release></releases>
<custom><value key="fs:signature">https://example.org/p.asc</value></custom></component></components>"#;

const DETAILS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<components version="1.0">
  <component type="desktop-application">
    <id>net.freesynergy.kanidm</id>
    <releases>
      <release version="1.5.0" date="2026-11-09">
        <description><p>Fixes the login loop.</p><ul><li>OIDC refresh</li></ul></description>
        <artifacts>
          <artifact type="binary" platform="x86_64-linux-gnu">
            <location>https://example.org/k.tar.gz</location>
            <checksum type="sha256">aa</checksum>
            <size type="download">13631488</size>
          </artifact>
        </artifacts>
      </release>
      <release version="1.4.0" date="2026-10-01"><description><p>Older.</p></description></release>
    </releases>
    <custom>
      <value key="fs:signature">https://example.org/k.asc</value>
      <value key="fs:upstream">https://github.com/kanidm/kanidm</value>
      <value key="fs:upstream-signature">https://example.org/k.upstream.asc</value>
      <value key="fs:source-code">https://example.org/k-src.tar.gz</value>
      <value key="fs:provides">identity.oidc 1.0, identity.accounts 1.1</value>
      <value key="fs:requires">people.persons >= 1.2, groups</value>
      <value key="fs:recommends">mail.send 1</value>
    </custom>
  </component>
</components>"#;

#[test]
fn details_of_the_chosen_release_are_read() {
    let c = parse_catalog(DETAILS).unwrap();
    let d = &c.entries[0].details;
    assert_eq!(d.size, Some(13_631_488));
    assert_eq!(d.released, "2026-11-09");
    assert_eq!(d.changes, "Fixes the login loop. OIDC refresh");
    assert!(d.origin.is_mirror());
    assert_eq!(d.origin.upstream, "https://github.com/kanidm/kanidm");
    assert_eq!(
        d.origin.upstream_signature,
        "https://example.org/k.upstream.asc"
    );
    assert_eq!(d.origin.source_code, "https://example.org/k-src.tar.gz");
    assert_eq!(d.provides.len(), 2);
    assert_eq!(d.requires[0].to_string(), "people.persons 1.2");
    assert_eq!(d.requires[1].to_string(), "groups");
    assert_eq!(d.recommends[0].to_string(), "mail.send 1.0");
    assert_eq!(d.unreadable.len(), 0);
}

#[test]
fn an_entry_without_details_has_empty_ones_and_unreadable_ones_are_named() {
    let c = parse_catalog(XML).unwrap();
    let d = &c.entries[0].details;
    assert_eq!(d.size, None);
    assert!(!d.origin.is_mirror());
    assert!(d.provides.is_empty() && d.requires.is_empty());
    let bad = DETAILS.replace("groups<", "gr oups x<");
    let c = parse_catalog(&bad).unwrap();
    assert_eq!(
        c.entries[0].details.unreadable,
        ["fs:requires: `gr oups x`"]
    );
}

const RICH: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<components version="1.0">
  <component type="desktop-application">
    <id>net.freesynergy.rich</id>
    <description><p>Long text.</p></description>
    <screenshots>
      <screenshot><image type="source" fs-sha256="h1" fs-license="CC-BY-4.0">https://example.org/1.png</image><caption>Main</caption></screenshot>
      <screenshot><image type="source" fs-sha256="h2">https://example.org/2.png</image></screenshot>
      <screenshot><image type="source" fs-sha256="h3" fs-license="Unknown">https://example.org/3.png</image></screenshot>
      <screenshot><image type="source" fs-license="MIT">https://example.org/4.png</image></screenshot>
      <screenshot><image type="source" fs-sha256="h5" fs-license="MIT">http://example.org/5.png</image></screenshot>
    </screenshots>
    <releases>
      <release version="1.0.0" date="2026-10-01"><description><p>First.</p></description></release>
      <release version="1.1.0" date="2026-11-01"><description><p>Second.</p></description>
        <artifacts><artifact type="binary"><location>https://example.org/r.tar.gz</location>
        <checksum type="sha256">r</checksum></artifact></artifacts></release>
    </releases>
    <custom><value key="fs:signature">https://example.org/r.asc</value>
    <value key="fs:description-wiki">wiki:rich-intro</value></custom>
  </component>
</components>"#;

#[test]
fn pictures_need_https_a_hash_and_a_clear_licence() {
    let d = &parse_catalog(RICH).unwrap().entries[0].details;
    assert_eq!(d.screenshots.len(), 1);
    assert_eq!(d.screenshots[0].license, "CC-BY-4.0");
    assert_eq!(d.screenshots[0].caption, "Main");
    assert_eq!(d.images_withheld.len(), 4);
    assert!(
        d.images_withheld
            .iter()
            .any(|w| w.contains("2.png") && w.contains("licence"))
    );
    assert!(
        d.images_withheld
            .iter()
            .any(|w| w.contains("3.png") && w.contains("licence"))
    );
    assert!(
        d.images_withheld
            .iter()
            .any(|w| w.contains("4.png") && w.contains("hash"))
    );
    assert!(
        d.images_withheld
            .iter()
            .any(|w| w.contains("5.png") && w.contains("https"))
    );
}

#[test]
fn changes_of_every_release_newest_first() {
    let d = &parse_catalog(RICH).unwrap().entries[0].details;
    let v: Vec<_> = d
        .releases
        .iter()
        .map(|r| (r.version.as_str(), r.text.as_str()))
        .collect();
    assert_eq!(v, [("1.1.0", "Second."), ("1.0.0", "First.")]);
}

#[test]
fn description_is_text_or_a_wiki_reference_and_trust_is_honest() {
    use fs_store_core::{Description, TrustLevel};
    let e = &parse_catalog(RICH).unwrap().entries[0];
    assert_eq!(
        e.details.description,
        Description::WikiBlock("wiki:rich-intro".into())
    );
    assert_eq!(e.description, "Long text.");
    assert_eq!(e.details.trust, Some(TrustLevel::OurKeyOnly));
    assert!(e.icon_missing());
    let k = &parse_catalog(DETAILS).unwrap().entries[0];
    assert_eq!(k.details.trust, Some(TrustLevel::MakerAndMirror));
    assert_eq!(k.details.description, Description::Text(String::new()));
}
