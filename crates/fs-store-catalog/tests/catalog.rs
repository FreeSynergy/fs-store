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
