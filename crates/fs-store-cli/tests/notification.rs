//! The desktop notification names programs FreeSynergy.Name, never fs-name.

use fs_store_sources::{Findings, Novelty, Update};

#[test]
fn the_notification_shows_display_names() {
    let findings = Findings {
        checked: 1,
        update: vec![Update {
            name: "fs-tasks".into(),
            from: "0.6.0".into(),
            to: "0.6.1".into(),
        }],
        new: vec![Novelty {
            name: "fs-people".into(),
            version: "0.0.3".into(),
        }],
        offered: Vec::new(),
    };
    assert_eq!(
        fs_store_cli::notification_lines(&findings),
        [
            "FreeSynergy.Tasks 0.6.0 -> 0.6.1",
            "New: FreeSynergy.People"
        ]
    );
}
