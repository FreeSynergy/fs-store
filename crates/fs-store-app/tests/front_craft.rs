//! What the Craft tiles and the footer say: the line under a badge, the progress of a running job (phases, not bytes), the footer.

use fs_store_app::demo::parts;
use fs_store_app::model::{CardState, StoreModel, apply_show_flag};

fn model() -> StoreModel {
    let p = parts(true);
    let mut m = StoreModel::new(p.backend).with_updates(p.updates);
    m.refresh();
    m
}

fn card(m: &StoreModel, name: &str) -> fs_store_app::model::Card {
    m.cards().into_iter().find(|c| c.name == name).unwrap()
}

#[test]
fn a_tile_tells_version_and_date_and_size_in_two_lines() {
    let m = model();
    let update = card(&m, "fs-tasks");
    assert_eq!(update.state, CardState::UpdateAvailable);
    assert_eq!(
        update.version_line_long(),
        "0.1.0 installed \u{2192} 0.2.0 released"
    );
    assert_eq!(update.meta_line(), "2026-11-09 - 14 MiB");
    let offered = card(&m, "fs-wiki");
    assert_eq!(offered.version_line_long(), "Version 0.1.4");
    assert_eq!(card(&m, "fs-mystery").meta_line(), "unknown - unknown");
    assert_eq!(update.state_word(), "Update available");
}

#[test]
fn a_running_job_shows_its_phase_as_a_share_never_as_bytes() {
    let mut m = model();
    assert!(card(&m, "fs-wiki").progress().is_none());
    apply_show_flag(&mut m, "--job", "fs-wiki:waiting").unwrap();
    let (share, line) = card(&m, "fs-wiki").progress().unwrap();
    assert!(share < 0.2 && line.contains("Waiting"));
    assert_eq!(card(&m, "fs-wiki").working_word(), "Waiting");
    apply_show_flag(&mut m, "--job", "fs-wiki:installing").unwrap();
    let (share, line) = card(&m, "fs-wiki").progress().unwrap();
    assert!(share > 0.5 && share < 1.0);
    assert_eq!(line, "Installing");
    assert!(apply_show_flag(&mut m, "--job", "fs-wiki:flying").is_err());
}

#[test]
fn the_footer_says_when_it_looked_and_what_it_installs() {
    let m = model();
    assert_eq!(m.footer_checked(), "Checked 3 min ago");
    assert_eq!(m.footer_format(), "Installing as tar.gz");
}
