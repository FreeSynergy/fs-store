//! Tests of the view-model: what the window starts on and which action each offer gets.

mod common;

use fs_store_app::backend::InstalledRow;
use fs_store_app::model::{Action, View};

use common::{action_of, fake, model, v};

#[test]
fn starts_on_cards_and_lists_actions() {
    let f = fake(
        vec![
            InstalledRow {
                name: "fs-tasks".into(),
                version: v(0, 6, 0),
            },
            InstalledRow {
                name: "fs-store".into(),
                version: v(0, 2, 0),
            },
        ],
        None,
    );
    let m = model(&f);
    assert_eq!(m.view(), View::Cards);
    assert_eq!(action_of(&m, "fs-tasks"), Some(Action::Update));
    assert_eq!(action_of(&m, "fs-store"), Some(Action::UpToDate));
    assert_eq!(action_of(&m, "fs-theme"), Some(Action::Install));
    assert_eq!(m.installed().len(), 2);
}
