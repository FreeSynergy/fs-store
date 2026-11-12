//! Tests of capabilities and the resolver (ADR 0068 point 1).

use fs_store_core::capability::{Capability, Provider, Resolution, resolve};

fn cap(t: &str) -> Capability {
    Capability::parse(t).unwrap()
}

fn provider(package: &str, provides: &[&str]) -> Provider {
    Provider {
        package: package.into(),
        provides: provides.iter().map(|t| cap(t)).collect(),
    }
}

#[test]
fn parses_the_forms_of_the_catalog() {
    assert_eq!(cap("people.persons").version, None);
    assert_eq!(cap("people.persons 1.2").version, Some((1, 2)));
    assert_eq!(cap("people.persons>=1.2").version, Some((1, 2)));
    assert_eq!(cap("people.persons >= 3").version, Some((3, 0)));
    assert_eq!(cap("  identity.oidc  ").name, "identity.oidc");
    assert_eq!(cap("people.persons 1.2").to_string(), "people.persons 1.2");
}

#[test]
fn refuses_what_is_not_a_capability() {
    assert!(Capability::parse("").is_none());
    assert!(Capability::parse("a/b").is_none());
    assert!(Capability::parse("people.persons x").is_none());
    assert!(Capability::parse("people.persons 1.x").is_none());
}

#[test]
fn a_list_reports_bad_entries() {
    let (good, bad) = Capability::parse_list("people.persons 1.2, , x y z, groups");
    assert_eq!(good.len(), 2);
    assert_eq!(bad, ["x y z"]);
}

#[test]
fn the_major_marks_a_break() {
    let offer = cap("people.persons 1.2");
    assert!(offer.satisfies(&cap("people.persons 1.0")));
    assert!(offer.satisfies(&cap("people.persons 1.2")));
    assert!(!offer.satisfies(&cap("people.persons 1.3")));
    assert!(!offer.satisfies(&cap("people.persons 2.0")));
    assert!(offer.satisfies(&cap("people.persons")));
    assert!(!cap("people.persons").satisfies(&cap("people.persons 1.0")));
    assert!(!offer.satisfies(&cap("people.users")));
}

#[test]
fn one_many_or_none() {
    let all = [
        provider("fs-people", &["people.persons 1.2", "people.users 1.0"]),
        provider("suite", &["people.persons 1.4"]),
        provider("old", &["people.persons 0.9"]),
    ];
    assert_eq!(
        resolve(&cap("people.users 1.0"), &all, "x"),
        Resolution::One("fs-people".into())
    );
    assert_eq!(
        resolve(&cap("people.persons 1.3"), &all, "x"),
        Resolution::One("suite".into())
    );
    assert_eq!(
        resolve(&cap("people.persons 1.0"), &all, "x"),
        Resolution::Choice(vec!["fs-people".into(), "suite".into()])
    );
    assert_eq!(resolve(&cap("groups"), &all, "x"), Resolution::Missing);
}

#[test]
fn a_package_does_not_answer_its_own_need() {
    let all = [provider("fs-people", &["people.persons 1.2"])];
    assert_eq!(
        resolve(&cap("people.persons"), &all, "fs-people"),
        Resolution::Missing
    );
}

#[test]
fn a_release_date_is_unix_milliseconds_at_midnight_or_nothing() {
    use fs_store_core::Details;
    assert_eq!(Details::millis_of("2026-11-09"), Some(1_794_182_400_000));
    assert_eq!(Details::millis_of("1970-01-01"), Some(0));
    assert_eq!(
        Details::millis_of("2026-11-09T10:00:00Z"),
        Some(1_794_182_400_000)
    );
    for bad in ["", "unknown", "2026-13-01", "2026-11", "2026-11-09-1"] {
        assert_eq!(Details::millis_of(bad), None, "{bad}");
    }
}
