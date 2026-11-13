//! The storefront model: navigation, filter tabs, search, sort, the table rows and the start page. Over the demo catalog,
//! which has every kind of fact in it (see `demo_data`).

use std::sync::Arc;

use fs_store_app::demo::DemoBackend;
use fs_store_app::model::{Layout, NeedsState, Section, Sort, StoreModel, Tab, apply_show_flag};
use fs_store_core::Kind;

fn model() -> StoreModel {
    let mut m = StoreModel::new(Arc::new(DemoBackend::default()));
    m.refresh();
    m
}

fn titles(m: &StoreModel) -> Vec<String> {
    m.listing().into_iter().map(|c| c.title).collect()
}

#[test]
fn the_navigation_counts_kinds_and_locks_what_has_nothing_or_cannot_be_said_yet() {
    let nav = model().nav();
    let by = |label: &str| nav.iter().find(|n| n.label == label).unwrap().clone();
    assert_eq!(by("Start").count, None);
    assert_eq!(
        (by("Programs").count, by("Programs").enabled),
        (Some(5), true)
    );
    assert_eq!(by("Frontends").count, Some(2));
    assert_eq!(by("Components").count, Some(2));
    assert_eq!(by("Themes").count, Some(1));
    // Icon sets: the catalog lists none, so the entry is locked; Servers: the demo lists one, so it opens, and both say "Coming soon"
    let icons = by("Icon sets");
    assert!(!icons.enabled && icons.section.is_none());
    assert_eq!(icons.note, Some("Coming soon"));
    let servers = by("Servers");
    assert_eq!((servers.count, servers.enabled), (Some(1), true));
    assert!(servers.section.is_some());
    assert_eq!(servers.note, Some("Coming soon"));
    assert!(by("Start").current);
}

#[test]
fn the_search_matches_nothing_without_hiding_the_kinds() {
    let mut m = model();
    m.set_search("zzz-nothing");
    // the nav counts packages, not matches: the kinds stay enabled
    assert!(
        m.nav()
            .iter()
            .filter(|n| n.section.is_some())
            .all(|n| n.enabled)
    );
    assert_eq!(m.listing().len(), 0);
}

#[test]
fn a_section_shows_one_kind_and_the_tabs_count_inside_it() {
    let mut m = model();
    m.open_section(Section::Kind(Kind::Program));
    assert_eq!(m.listing().len(), 5);
    let counts = m.tab_counts();
    assert_eq!(counts[0], (Tab::All, 5));
    assert_eq!(counts[1], (Tab::Installed, 2));
    assert_eq!(counts[2], (Tab::Updates, 1));
    assert_eq!(counts[3], (Tab::NotInstalled, 3));
    m.set_tab(Tab::Updates);
    assert_eq!(titles(&m), ["FreeSynergy.Tasks"]);
    m.set_tab(Tab::NotInstalled);
    assert_eq!(m.listing().len(), 3);
}

#[test]
fn the_search_narrows_by_title_name_or_description_and_ignores_case() {
    let mut m = model();
    m.set_search("CONTACTS");
    assert_eq!(titles(&m), ["FreeSynergy.People"]);
    m.set_search("kanidm");
    assert_eq!(titles(&m), ["Kanidm"]);
    m.set_search("fs-theme");
    assert_eq!(titles(&m), ["Dark theme"]);
    m.set_search("");
    assert_eq!(m.listing().len(), 12);
}

#[test]
fn sorting_puts_what_the_catalog_does_not_say_last() {
    let mut m = model();
    m.set_sort(Sort::Size);
    let sizes: Vec<String> = m.table_rows().into_iter().map(|r| r.size).collect();
    assert_eq!(sizes.first().unwrap(), "1.2 MiB");
    assert_eq!(sizes.last().unwrap(), "unknown");
    m.set_sort(Sort::Newest);
    let rows = m.table_rows();
    assert_eq!(rows[0].released, "2026-11-11");
    assert_eq!(rows.last().unwrap().released, "unknown");
    m.set_sort(Sort::Name);
    assert_eq!(titles(&m)[0], "Dark theme");
}

#[test]
fn the_table_says_unknown_where_the_catalog_is_silent_and_never_guesses() {
    let m = model();
    let rows = m.table_rows();
    let mystery = rows.iter().find(|r| r.card.name == "fs-mystery").unwrap();
    assert_eq!(mystery.released, "unknown");
    assert_eq!(mystery.size, "unknown");
    assert_eq!(mystery.signature, "unknown");
    assert_eq!(mystery.installed, "-");
    let kanidm = rows.iter().find(|r| r.card.name == "kanidm").unwrap();
    assert_eq!(kanidm.signature, "Maker + mirror (catalog)");
    assert_eq!(kanidm.size, "38 MiB");
    assert_eq!(
        (kanidm.installed.as_str(), kanidm.latest.as_str()),
        ("1.4.2", "1.5.0")
    );
    let local = rows
        .iter()
        .find(|r| r.card.name == "fs-local-test")
        .unwrap();
    assert_eq!(local.signature, "Checksum only");
}

#[test]
fn needs_are_answered_by_the_resolver_one_choice_or_missing() {
    let m = model();
    let rows = m.table_rows();
    let state = |n: &str| rows.iter().find(|r| r.card.name == n).unwrap().needs_state;
    // wiki needs texts.blocks, nobody offers it
    assert_eq!(state("fs-wiki"), NeedsState::Missing);
    // tasks needs people.persons (met) and groups (nobody)
    assert_eq!(state("fs-tasks"), NeedsState::Missing);
    // stalwart needs identity.accounts, kanidm offers it
    assert_eq!(state("stalwart"), NeedsState::Met);
    assert_eq!(state("fs-people"), NeedsState::None);
    let l = m.capability_lines("fs-tasks");
    assert_eq!(l.requires[0].answer, "provided by FreeSynergy.People");
    assert_eq!(l.requires[1].answer, "nobody offers it");
    assert_eq!(l.provides[0].text, "tasks.items 1.0");
}

#[test]
fn the_start_page_is_an_overview_and_the_lists_are_the_sections() {
    let m = model();
    let blocks = m.start_blocks();
    assert_eq!(blocks[0].title, "Updates");
    assert_eq!(blocks[0].total, 2);
    // counter-proof: no block repeats a kind list (that is what the sections are for)
    assert!(blocks.iter().all(|b| b.title != "Programs" && b.more.is_none()));
}

#[test]
fn every_section_counts_exactly_what_it_lists_and_the_start_has_no_number() {
    let mut m = model();
    let nav = m.nav();
    assert_eq!(nav[0].count, None);
    for entry in nav.iter().skip(1).filter(|n| n.enabled) {
        m.open_section(entry.section.unwrap());
        assert_eq!(Some(m.listing().len()), entry.count, "{}", entry.label);
        assert_eq!(m.section_title(), entry.label);
    }
    // counter-proof: the sections together hold every card of the catalog, none twice and none left out; the one kind the store does
    // not know is shown apart on the start page
    let sum: usize = nav.iter().filter_map(|n| n.count).sum();
    let unknown = m
        .cards()
        .iter()
        .filter(|c| c.kind == Some(Kind::Unsupported))
        .count();
    assert_eq!(unknown, 1);
    assert_eq!(sum + unknown, m.cards().len());
    // and a number that is not the list would be caught: the start shows all cards as a list only when a filter asks for it
    m.open_section(Section::Start);
    assert_ne!(Some(m.listing().len()), nav[1].count);
}

#[test]
fn escape_closes_the_question_then_the_page_then_goes_to_the_start() {
    let mut m = model();
    m.open_section(Section::Kind(Kind::Component));
    m.open_page("kanidm");
    m.ask_install("kanidm");
    assert!(m.preview().is_some());
    assert!(m.escape());
    assert!(m.preview().is_none() && m.page_name().is_none() || m.page_name() == Some("kanidm"));
    while m.escape() {}
    assert_eq!(m.front().section(), Section::Start);
    assert!(!m.escape());
}

#[test]
fn layout_switches_and_a_start_flag_that_is_not_a_choice_is_refused() {
    let mut m = model();
    assert_eq!(m.front().layout(), Layout::Grid);
    m.toggle_layout();
    assert_eq!(m.front().layout(), Layout::Table);
    apply_show_flag(&mut m, "--layout", "grid").unwrap();
    assert_eq!(m.front().layout(), Layout::Grid);
    apply_show_flag(&mut m, "--section", "frontend").unwrap();
    assert_eq!(m.listing().len(), 2);
    assert!(apply_show_flag(&mut m, "--layout", "cube").is_err());
}

#[test]
fn the_footer_names_the_sources_from_the_settings() {
    let m = model();
    assert_eq!(
        m.footer_sources(),
        "Sources (from the settings): freesynergy"
    );
    assert_eq!(m.footer_count(), "12 packages");
}

#[test]
fn a_long_summary_is_closed_with_an_ellipsis_at_a_word_never_cut_hard() {
    use fs_store_app::model::{ellipsize, summary};
    assert_eq!(
        ellipsize("Identity management: accounts, groups", 40),
        "Identity management: accounts, groups"
    );
    assert_eq!(
        ellipsize("Identity management: accounts, groups and sign-in", 31),
        "Identity management: accounts\u{2026}"
    );
    assert_eq!(
        ellipsize("Identity management: accounts, groups", 22),
        "Identity management\u{2026}"
    );
    assert_eq!(ellipsize("Supercalifragilistic", 8), "Superca\u{2026}");
    assert_eq!(
        summary("First sentence is here. Second one.", 100),
        "First sentence is here."
    );
    assert!(summary("A long first sentence that does not fit at all.", 20).ends_with('\u{2026}'));
}
