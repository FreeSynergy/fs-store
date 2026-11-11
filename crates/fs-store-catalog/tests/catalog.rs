use fs_store_catalog::{Skipped, parse_catalog};
use fs_store_core::{Format, Kind, Version};

const XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<components version="1.0" origin="freesynergy">
  <component type="desktop-application">
    <id>net.freesynergy.tasks</id>
    <name>Tasks</name>
    <summary>Tasks and Kanban</summary>
    <releases>
      <release version="0.7.0" date="2026-11-01"/>
      <release version="0.8.0" date="2026-11-07">
        <artifacts>
          <artifact type="binary" platform="x86_64-linux-gnu">
            <location>https://example.org/tasks-0.8.0.tar.gz</location>
            <checksum type="sha256">abc123</checksum>
          </artifact>
        </artifacts>
      </release>
    </releases>
    <custom>
      <value key="fs:kind">program</value>
      <value key="fs:format">tar.gz</value>
      <value key="fs:signature">https://example.org/tasks-0.8.0.tar.gz.asc</value>
      <value key="fs:facets">kanban, list</value>
    </custom>
  </component>
  <component type="addon">
    <id>net.freesynergy.theme.dark</id>
    <name>Dark</name>
    <releases><release version="1.0.0"><artifacts><artifact type="binary">
      <location>https://example.org/dark.tar.gz</location><checksum type="sha256">d4</checksum>
    </artifact></artifacts></release></releases>
    <custom><value key="fs:kind">theme</value><value key="fs:signature">https://example.org/dark.asc</value></custom>
  </component>
  <component type="desktop-application">
    <id>net.freesynergy.pre</id>
    <releases><release version="0.6.1~nightly"><artifacts><artifact type="binary">
      <location>https://example.org/pre.tar.gz</location><checksum type="sha256">p</checksum>
    </artifact></artifacts></release></releases>
    <custom><value key="fs:signature">https://example.org/pre.asc</value></custom>
  </component>
  <component type="desktop-application">
    <id>net.freesynergy.nosig</id>
    <releases><release version="1.0.0"><artifacts><artifact type="binary">
      <location>https://example.org/n.tar.gz</location><checksum type="sha256">n</checksum>
    </artifact></artifacts></release></releases>
  </component>
</components>"#;

#[test]
fn newest_release_with_artifact_is_offered() {
    let c = parse_catalog(XML).unwrap();
    let t = &c.entries[0];
    assert_eq!(t.name.as_str(), "net.freesynergy.tasks");
    assert_eq!(t.kind, Kind::Program);
    assert_eq!(t.format, Format::TarGz);
    assert_eq!(t.version, Version::new(0, 8, 0));
    assert_eq!(t.location, "https://example.org/tasks-0.8.0.tar.gz");
    assert_eq!(t.sha256, "abc123");
    assert_eq!(
        t.signature_url,
        "https://example.org/tasks-0.8.0.tar.gz.asc"
    );
    assert_eq!(t.facets, ["kanban", "list"]);
}

#[test]
fn kind_defaults_to_program_and_theme_is_read() {
    let c = parse_catalog(XML).unwrap();
    assert_eq!(c.entries[1].kind, Kind::Theme);
}

#[test]
fn unusable_components_are_reported_never_dropped_silently() {
    let c = parse_catalog(XML).unwrap();
    assert_eq!(c.entries.len(), 2);
    assert_eq!(c.skipped.len(), 2);
    assert!(c.skipped.contains(&Skipped {
        id: "net.freesynergy.pre".into(),
        reason: "version `0.6.1~nightly` is not major.minor.patch".into()
    }));
    assert!(
        c.skipped
            .iter()
            .any(|s| s.id == "net.freesynergy.nosig" && s.reason.contains("signature"))
    );
}

#[test]
fn broken_xml_is_an_error() {
    assert!(parse_catalog("<components>").is_err());
    assert!(parse_catalog("<other/>").is_err());
}

#[test]
fn bad_old_release_and_foreign_arch_do_not_hide_a_good_release_and_duplicates_are_reported() {
    let xml = r#"<components version="1.0">
<component><id>a</id><releases>
 <release version="2024.1"><artifacts><artifact type="binary"><location>l0</location><checksum type="sha256">0</checksum></artifact></artifacts></release>
 <release version="1.0.0"><artifacts>
   <artifact type="binary" platform="aarch64-linux-gnu"><location>arm</location><checksum type="sha256">1</checksum></artifact>
   <artifact type="binary" platform="x86_64-linux-gnu"><location>x86</location><checksum type="sha256">2</checksum></artifact>
 </artifacts></release></releases>
 <custom><value key="fs:signature">s</value></custom></component>
<component><id>a</id><releases><release version="1.0.0"><artifacts><artifact type="binary"><location>l</location><checksum type="sha256">3</checksum></artifact></artifacts></release></releases>
 <custom><value key="fs:signature">s</value></custom></component>
<component><id>../evil</id><releases><release version="1.0.0"><artifacts><artifact type="binary"><location>l</location><checksum type="sha256">3</checksum></artifact></artifacts></release></releases>
 <custom><value key="fs:signature">s</value></custom></component>
</components>"#;
    let c = parse_catalog(xml).unwrap();
    assert_eq!(c.entries.len(), 1);
    if std::env::consts::ARCH == "x86_64" {
        assert_eq!(c.entries[0].location, "x86");
    }
    assert!(c.skipped.iter().any(|s| s.reason.contains("duplicate")));
    assert!(c.skipped.iter().any(|s| s.id == "../evil"));
}

#[test]
fn serial_and_expiry_are_read_when_present() {
    let c = parse_catalog(r#"<components version="1.0" fs-serial="42" fs-expires="1790000000"/>"#)
        .unwrap();
    assert_eq!((c.serial, c.expires), (Some(42), Some(1_790_000_000)));
    let c = parse_catalog(r#"<components version="1.0"/>"#).unwrap();
    assert_eq!((c.serial, c.expires), (None, None));
}

#[test]
fn rpm_artifacts_are_collected_and_do_not_replace_the_primary_tar_gz() {
    let xml = r#"<components version="1.0"><component><id>t</id><releases><release version="1.0.0"><artifacts>
 <artifact type="binary" platform="x86_64-linux-gnu"><location>tgz</location><checksum type="sha256">1</checksum></artifact>
 <artifact type="binary" platform="x86_64-linux-gnu" fs-format="rpm" fs-signature="app.asc"><location>app.rpm</location><checksum type="sha256">2</checksum></artifact>
 <artifact type="binary" platform="x86_64-linux-gnu" fs-format="rpm" fs-signature="srv.asc"><location>srv.rpm</location><checksum type="sha256">3</checksum></artifact>
 <artifact type="binary" platform="x86_64-linux-gnu" fs-format="rpm"><location>unsigned.rpm</location><checksum type="sha256">4</checksum></artifact>
 </artifacts></release></releases><custom><value key="fs:signature">tgz.asc</value></custom></component></components>"#;
    let c = parse_catalog(xml).unwrap();
    let e = &c.entries[0];
    assert_eq!(e.location, "tgz");
    let names: Vec<_> = e.rpms.iter().map(|r| r.location.as_str()).collect();
    if std::env::consts::ARCH == "x86_64" {
        assert_eq!(names, ["app.rpm", "srv.rpm"]); // the unsigned rpm is never offered
    }
}

#[test]
fn a_remote_icon_needs_a_hash_to_be_used() {
    let with = r#"<components version="1.0"><component><id>a</id><icon type="remote" fs-sha256="ab">https://x/a.png</icon><releases><release version="1.0.0"><artifacts><artifact type="binary"><location>l</location><checksum type="sha256">1</checksum></artifact></artifacts></release></releases><custom><value key="fs:signature">s</value></custom></component></components>"#;
    let c = parse_catalog(with).unwrap();
    let icon = c.entries[0].icon.as_ref().unwrap();
    assert_eq!(
        (icon.url.as_str(), icon.sha256.as_str()),
        ("https://x/a.png", "ab")
    );
    let without = with.replace(r#" fs-sha256="ab""#, "");
    assert!(parse_catalog(&without).unwrap().entries[0].icon.is_none());
}

#[test]
fn description_and_homepage_are_read() {
    let xml = r#"<components version="1.0"><component><id>a</id><url type="homepage"> https://github.com/FreeSynergy/fs-a </url>
<description><p>Tasks   and
 more.</p><p>Second.</p></description><releases><release version="1.0.0"><artifacts><artifact type="binary"><location>l</location><checksum type="sha256">1</checksum></artifact></artifacts></release></releases><custom><value key="fs:signature">s</value></custom></component></components>"#;
    let e = &parse_catalog(xml).unwrap().entries[0];
    assert_eq!(e.description, "Tasks and more. Second.");
    assert_eq!(e.homepage, "https://github.com/FreeSynergy/fs-a");
}

#[test]
fn the_summary_stands_in_when_there_is_no_description() {
    let xml = r#"<components version="1.0"><component><id>a</id><summary>One line.</summary><releases><release version="1.0.0"><artifacts><artifact type="binary"><location>l</location><checksum type="sha256">1</checksum></artifact></artifacts></release></releases><custom><value key="fs:signature">s</value></custom></component></components>"#;
    assert_eq!(
        parse_catalog(xml).unwrap().entries[0].description,
        "One line."
    );
}

#[test]
fn deb_and_exe_artifacts_are_collected_by_platform_and_never_become_the_primary_file() {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let xml = format!(
        r#"<components version="1.0"><component><id>t</id><releases><release version="1.0.0"><artifacts>
 <artifact type="binary" platform="{arch}-{os}"><location>tgz</location><checksum type="sha256">1</checksum></artifact>
 <artifact type="binary" platform="{arch}-{os}" fs-format="deb" fs-signature="a.asc"><location>app.deb</location><checksum type="sha256">2</checksum></artifact>
 <artifact type="binary" platform="{arch}-{os}" fs-format="exe" fs-signature="s.asc"><location>setup.exe</location><checksum type="sha256">3</checksum></artifact>
 <artifact type="binary" platform="{arch}-other" fs-format="deb" fs-signature="b.asc"><location>other.deb</location><checksum type="sha256">4</checksum></artifact>
 <artifact type="binary" platform="{arch}-{os}" fs-format="deb"><location>unsigned.deb</location><checksum type="sha256">5</checksum></artifact>
 </artifacts></release></releases><custom><value key="fs:signature">tgz.asc</value></custom></component></components>"#
    );
    let e = &parse_catalog(&xml).unwrap().entries[0];
    assert_eq!(e.location, "tgz");
    let debs: Vec<_> = e.debs.iter().map(|r| r.location.as_str()).collect();
    let exes: Vec<_> = e.exes.iter().map(|r| r.location.as_str()).collect();
    assert_eq!(debs, ["app.deb"]); // not for another system, not unsigned
    assert_eq!(exes, ["setup.exe"]); // this machine's platform says so; on Linux the platform of a Windows file does not
}

#[test]
fn a_windows_file_is_not_offered_on_linux() {
    let xml = r#"<components version="1.0"><component><id>t</id><releases><release version="1.0.0"><artifacts>
 <artifact type="binary" platform="x86_64-linux-gnu"><location>tgz</location><checksum type="sha256">1</checksum></artifact>
 <artifact type="binary" platform="x86_64-windows-msvc" fs-format="exe" fs-signature="s.asc"><location>setup.exe</location><checksum type="sha256">3</checksum></artifact>
 </artifacts></release></releases><custom><value key="fs:signature">tgz.asc</value></custom></component></components>"#;
    let e = &parse_catalog(xml).unwrap().entries[0];
    if std::env::consts::OS == "linux" {
        assert_eq!(e.exes, []);
    }
}

#[test]
fn the_display_name_is_read_from_the_name_tag() {
    let xml = r#"<components version="1.0"><component><id>fs-a</id><name> FreeSynergy.A </name><releases><release version="1.0.0"><artifacts><artifact type="binary"><location>l</location><checksum type="sha256">1</checksum></artifact></artifacts></release></releases><custom><value key="fs:signature">s</value></custom></component></components>"#;
    assert_eq!(
        parse_catalog(xml).unwrap().entries[0].title,
        "FreeSynergy.A"
    );
}

#[test]
fn the_name_without_a_language_wins() {
    let xml = r#"<components version="1.0"><component><id>fs-a</id><name xml:lang="de">Anders</name><name>FreeSynergy.A</name><releases><release version="1.0.0"><artifacts><artifact type="binary"><location>l</location><checksum type="sha256">1</checksum></artifact></artifacts></release></releases><custom><value key="fs:signature">s</value></custom></component></components>"#;
    assert_eq!(
        parse_catalog(xml).unwrap().entries[0].title,
        "FreeSynergy.A"
    );
}

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
    assert!(d.unreadable.is_empty());
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
