//! Pattern: Composite (the admin page is one column of sections: the update choices, the daily timer, the
//! programs with their switch, the sources). The menu entry "Sources" opens the same page with that section
//! first, because Blinc has no scroll-to.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::model::StoreModel;
use crate::words;

use super::settings::{choice, section, updates_section};
use super::shell::{Shared, locked};
use super::sources::sources_section;

/// The daily timer: its state, and the two buttons that call `fs-store timer enable|disable`.
fn timer_section(model: &StoreModel, shared: &Shared) -> Div {
    let state = if model.timer_on() {
        words::TIMER_IS_ON
    } else {
        words::TIMER_IS_OFF
    };
    let mut row = div().class("fs-store-add");
    for (on, label) in [
        (true, words::BUTTON_TIMER_ON),
        (false, words::BUTTON_TIMER_OFF),
    ] {
        let shared = Arc::clone(shared);
        row = row.child(choice(
            &format!("timer-{on}"),
            label,
            model.timer_on() == on,
            move || locked(&shared).set_timer(on),
        ));
    }
    let mut block = section(words::TIMER_HEADING, row).child(text(state).class("fs-store-sub"));
    if let Some(note) = model.admin_note() {
        block = block.child(text(note.to_owned()).class("fs-store-sub"));
    }
    block
}

/// One program and its On/Off pair for automatic updates.
fn programs_section(model: &StoreModel, shared: &Shared) -> Div {
    let mut list = div().flex_col();
    if model.programs().is_empty() {
        list = list.child(text(words::PROGRAMS_NONE).class("fs-store-sub"));
    }
    for program in model.programs() {
        let mut pair = div().class("fs-store-add");
        for (on, label) in [(true, words::NOTIFY_ON), (false, words::NOTIFY_OFF)] {
            let (shared, name) = (Arc::clone(shared), program.name.clone());
            pair = pair.child(choice(
                &format!("auto-{}-{on}", program.name),
                label,
                program.auto_update == on,
                move || locked(&shared).set_program_auto(&name, on),
            ));
        }
        list = list.child(
            div()
                .class("fs-store-row")
                .child(
                    div()
                        .class("fs-store-cell")
                        .child(text(model.title(&program.name))),
                )
                .child(pair),
        );
    }
    section(words::PROGRAMS_HEADING, list)
}

fn sources_block(model: &StoreModel, shared: &Shared) -> Div {
    div()
        .flex_col()
        .class("fs-store-section")
        .child(text(words::SOURCES_HEADING).bold())
        .child(sources_section(model, shared))
}

/// The whole admin page. `sources_first` puts the source list on top (the menu entry "Sources").
pub(super) fn admin(model: &StoreModel, shared: &Shared, sources_first: bool) -> Div {
    let mut page = div().flex_col().w_full();
    if sources_first {
        page = page.child(sources_block(model, shared));
    }
    page = page
        .child(updates_section(model, shared))
        .child(timer_section(model, shared))
        .child(programs_section(model, shared));
    if !sources_first {
        page = page.child(sources_block(model, shared));
    }
    page
}
