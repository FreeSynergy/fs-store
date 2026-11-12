//! Pattern: Adapter (the "more" menu of a card: three dots one above the other, and a drop-down with
//! "Show description", "Go to the Git repository" and "Uninstall").

use std::sync::Arc;

use fs_components::blinc::cn::components::dropdown_menu::{DropdownAlign, dropdown_menu_custom};
use fs_components::blinc::layout::ElementBuilder;
use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;

use crate::launcher::Launcher;
use crate::model::{Card, CardState};
use crate::words;

use super::shell::{Shared, locked, rebuild};

/// The trigger: a small square with the three dots one above the other.
fn trigger(open: bool) -> Div {
    let mut square = div()
        .class("fs-more")
        .w(32.0)
        .h(32.0)
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .cursor_pointer();
    if open {
        square = square.class("fs-more-open");
    }
    square.child(text(words::MORE_DOTS).size(20.0).bold())
}

/// The three-dot menu of `card`.
pub(super) fn more_menu(
    card: &Card,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
) -> impl ElementBuilder + use<> {
    let (describe, repository, remove) =
        (Arc::clone(shared), Arc::clone(shared), Arc::clone(shared));
    let launcher = Arc::clone(launcher);
    let (n1, n2, n3) = (card.name.clone(), card.name.clone(), card.name.clone());
    let menu = dropdown_menu_custom(trigger)
        .align(DropdownAlign::End)
        .min_width(220.0)
        .item(words::MORE_DESCRIPTION, move || {
            locked(&describe).open_page(&n1);
            rebuild();
        })
        .item(words::MORE_REPOSITORY, move || {
            locked(&repository).open_repository(&n2, launcher.as_ref());
            rebuild();
        })
        .separator();
    if card.state == CardState::NotInstalled {
        menu.item_disabled(words::MORE_UNINSTALL)
    } else {
        menu.item(words::MORE_UNINSTALL, move || {
            locked(&remove).ask_remove_preview(&n3);
            rebuild();
        })
    }
}
