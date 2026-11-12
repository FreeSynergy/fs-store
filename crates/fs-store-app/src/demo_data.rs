//! Pattern: none (sample catalog for `--demo`, the acceptance pictures and the tests of the window). Every fact the
//! storefront can show is in here at least once: sizes, dates, all three trust levels, capabilities that are met, need a
//! choice or are missing, pictures with and without a licence, a description that points to the wiki, a package with no
//! facts at all. The facts are made up for the demo; the real window reads them from the signed catalog.

use fs_store_core::{
    Capability, Description, Details, Kind, Origin, ReleaseNote, Screenshot, TrustLevel, Version,
};

use crate::backend::{InstalledRow, PackageRow};

fn caps(text: &str) -> Vec<Capability> {
    Capability::parse_list(text).0
}

#[allow(clippy::too_many_arguments)]
fn row(
    (name, title): (&str, &str),
    kind: Kind,
    version: (u64, u64, u64),
    summary: &str,
    mib: Option<u64>,
    released: &str,
    trust: Option<TrustLevel>,
) -> PackageRow {
    PackageRow {
        name: name.into(),
        title: title.into(),
        kind,
        version: Version::new(version.0, version.1, version.2),
        source: "freesynergy".into(),
        description: summary.into(),
        homepage: "https://example.invalid".into(),
        native: Vec::new(),
        locked: None,
        details: Details {
            size: mib.map(|m| m * 1024 * 1024 + 300_000),
            released: released.into(),
            trust,
            origin: Origin {
                source: "https://example.invalid/freesynergy".into(),
                ..Origin::default()
            },
            description: Description::Text(summary.into()),
            ..Details::default()
        },
    }
}

fn note(version: &str, date: &str, text: &str) -> ReleaseNote {
    ReleaseNote {
        version: version.into(),
        date: date.into(),
        text: text.into(),
    }
}

/// Our own programs.
fn programs() -> Vec<PackageRow> {
    use TrustLevel::{ChecksumOnly, OurKeyOnly};
    let mut tasks = row(
        ("fs-tasks", "FreeSynergy.Tasks"),
        Kind::Program,
        (0, 2, 0),
        "Tasks for people and groups. Boards, lists and a plan you can share.",
        Some(14),
        "2026-11-09",
        Some(OurKeyOnly),
    );
    tasks.details.provides = caps("tasks.items 1.0");
    tasks.details.requires = caps("people.persons 1.2, groups");
    tasks.details.recommends = caps("mail.send 1.0");
    tasks.details.releases = vec![
        note(
            "0.2.0",
            "2026-11-09",
            "Boards can be shared with a group. The list view keeps its place after an update.",
        ),
        note("0.1.0", "2026-10-21", "First release."),
    ];
    tasks.details.screenshots = vec![Screenshot {
        url: "https://example.invalid/tasks-board.png".into(),
        sha256: "00".into(),
        caption: "The board".into(),
        source: "https://example.invalid/maker-shots".into(),
        license: "CC-BY-4.0".into(),
    }];
    tasks.details.images_withheld = vec!["https://example.invalid/x.png: no clear licence".into()];
    let mut people = row(
        ("fs-people", "FreeSynergy.People"),
        Kind::Program,
        (0, 3, 1),
        "Contacts and members. One record per person, with the roles they have.",
        Some(11),
        "2026-11-05",
        Some(OurKeyOnly),
    );
    people.details.provides = caps("people.persons 1.4, people.users 1.0");
    people.details.releases = vec![note(
        "0.3.1",
        "2026-11-05",
        "The e-mail field keeps the dot.",
    )];
    let mut wiki = row(
        ("fs-wiki", "FreeSynergy.Wiki"),
        Kind::Program,
        (0, 1, 4),
        "Notes that belong together. Pages, links and templates.",
        Some(9),
        "2026-10-30",
        Some(OurKeyOnly),
    );
    wiki.details.requires = caps("texts.blocks 1.0");
    wiki.details.description = Description::WikiBlock(fs_store_core::WikiRef {
        block: "0190a1f2-7c3b-7d10-9a55-2f4e8b6c1d30".into(),
        source_rev: Some(14),
        lang: String::new(),
    });
    let mut mystery = row(
        ("fs-mystery", ""),
        Kind::Program,
        (0, 0, 1),
        "",
        None,
        "",
        None,
    );
    mystery.details.description = Description::Text(String::new());
    mystery.homepage.clear();
    let mut local = row(
        ("fs-local-test", "Local test build"),
        Kind::Program,
        (0, 0, 2),
        "A build from a local source, checked only by its checksum.",
        Some(3),
        "2026-11-11",
        Some(ChecksumOnly),
    );
    local.details.provides = caps("tasks.items 1.1");
    tasks.details.installed_size = Some(41_943_040);
    tasks.details.origin.maker = "FreeSynergy".into();
    tasks.details.mediatypes = vec!["text/calendar".into()];
    vec![tasks, people, wiki, mystery, local]
}

/// Entries this store version lists but cannot install: a kind it does not install yet, and a kind it does not know.
fn locked() -> Vec<PackageRow> {
    use TrustLevel::OurKeyOnly;
    let mut relay = row(
        ("fs-relay", "FreeSynergy.Relay"),
        Kind::Server,
        (0, 1, 0),
        "A background service that relays messages between programs.",
        None,
        "",
        Some(OurKeyOnly),
    );
    relay.locked = Some("coming soon".into());
    let mut odd = row(
        ("fs-hologram", "Hologram"),
        Kind::Unsupported,
        (1, 0, 0),
        "A kind of package that a newer store knows.",
        None,
        "",
        Some(OurKeyOnly),
    );
    odd.locked = Some("not supported by this store version".into());
    vec![relay, odd]
}

/// Foreign components, mirrored: the maker's signature and the mirror's.
fn components() -> Vec<PackageRow> {
    use TrustLevel::MakerAndMirror;
    let mut kanidm = row(
        ("kanidm", "Kanidm"),
        Kind::Component,
        (1, 5, 0),
        "Identity management: accounts, groups and sign-in for all programs.",
        Some(38),
        "2026-11-08",
        Some(MakerAndMirror),
    );
    kanidm.details.origin.upstream = "https://github.com/kanidm/kanidm".into();
    kanidm.details.origin.upstream_signature = "https://example.invalid/kanidm.upstream.asc".into();
    kanidm.details.origin.source_code = "https://example.invalid/kanidm-1.5.0-src.tar.gz".into();
    kanidm.details.provides = caps("identity.oidc 1.0, identity.accounts 1.1");
    kanidm.details.requires = caps("people.persons >= 1.2");
    kanidm.details.recommends = caps("mail.send 1.0");
    kanidm.details.releases = vec![
        note(
            "1.5.0",
            "2026-11-08",
            "Fixes the sign-in loop after a token refresh.",
        ),
        note("1.4.2", "2026-10-02", ""),
    ];
    let mut stalwart = row(
        ("stalwart", "Stalwart"),
        Kind::Component,
        (0, 11, 2),
        "Mail server: sends, receives and keeps mail.",
        Some(52),
        "2026-10-18",
        Some(MakerAndMirror),
    );
    stalwart.details.origin.upstream = "https://github.com/stalwartlabs/stalwart".into();
    stalwart.details.origin.upstream_signature =
        "https://example.invalid/stalwart.upstream.asc".into();
    stalwart.details.origin.source_code =
        "https://example.invalid/stalwart-0.11.2-src.tar.gz".into();
    stalwart.details.provides = caps("mail.send 1.0, mail.receive 1.0");
    stalwart.details.requires = caps("identity.accounts 1.0");
    vec![kanidm, stalwart]
}

/// Frontends and themes.
fn looks() -> Vec<PackageRow> {
    use TrustLevel::OurKeyOnly;
    let mut admin = row(
        ("fs-admin-frontend", "FreeSynergy.Admin"),
        Kind::Frontend,
        (0, 1, 0),
        "The admin look for every program: menus, tables and forms in one style.",
        Some(6),
        "2026-11-07",
        Some(OurKeyOnly),
    );
    admin.details.requires = caps("frontend.engine 2.0");
    let mut files_view = row(
        ("fs-files-view", "FreeSynergy.Files View"),
        Kind::Frontend,
        (0, 0, 3),
        "A file window like the ones you know: places, list and grid, preview.",
        Some(7),
        "2026-11-10",
        Some(OurKeyOnly),
    );
    files_view.details.releases = vec![note("0.0.3", "2026-11-10", "Grid view with big icons.")];
    let dark = row(
        ("fs-theme-dark", "Dark theme"),
        Kind::Theme,
        (1, 0, 0),
        "A dark look for all windows.",
        Some(1),
        "2026-09-14",
        Some(OurKeyOnly),
    );
    vec![admin, files_view, dark]
}

/// The offered packages.
pub(crate) fn offered() -> Vec<PackageRow> {
    let mut all = programs();
    all.extend(components());
    all.extend(looks());
    all.extend(locked());
    all
}

/// What is installed: one up to date, two with a newer version offered.
pub(crate) fn installed() -> Vec<InstalledRow> {
    let at = |name: &str, v: Version| InstalledRow {
        name: name.into(),
        version: v,
    };
    vec![
        at("fs-tasks", Version::new(0, 1, 0)),
        at("fs-people", Version::new(0, 3, 1)),
        at("kanidm", Version::new(1, 4, 2)),
    ]
}
