//! Pattern: Adapter (the updates screen: one row per update with the existing update job behind its button,
//! then the programs marked new) and the check run on a worker thread.

use std::sync::Arc;

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::model::{StoreModel, update_line};
use crate::words;

use super::job::start_job;
use super::shell::{Shared, button, locked, rebuild};

pub(super) fn updates(model: &StoreModel, shared: &Shared) -> Div {
    let mut page = div().flex_col().w_full().child(
        div()
            .class("fs-store-empty")
            .child(text(words::UPDATES_HEADING)),
    );
    if model.update_list().is_empty() {
        page = page.child(
            div()
                .class("fs-store-empty")
                .child(text(words::UPDATES_NONE)),
        );
    }
    for item in model.update_list() {
        let (shared, name) = (Arc::clone(shared), item.name.clone());
        page = page.child(
            div()
                .class("fs-store-row")
                .child(div().class("fs-store-cell").child(text(update_line(item))))
                .child(
                    button(
                        &format!("update-{}", item.name),
                        words::BUTTON_UPDATE,
                        ButtonVariant::Primary,
                        !model.is_busy(),
                    )
                    .on_click(move |_| start_job(&shared, &name)),
                ),
        );
    }
    if !model.new_programs().is_empty() {
        page = page.child(
            div()
                .class("fs-store-empty")
                .child(text(words::NEW_HEADING)),
        );
    }
    for novelty in model.new_programs() {
        let line = words::fill(
            words::NEW_LINE,
            &[("name", &novelty.name), ("version", &novelty.version)],
        );
        page = page.child(
            div()
                .class("fs-store-row")
                .child(div().class("fs-store-cell").child(text(line))),
        );
    }
    page
}

/// Looks for updates once, on a worker thread (the check is a child process of its own); the window is asked
/// to draw again when it starts and when it ends. Does nothing if a check runs already.
pub fn start_check(shared: &Shared) {
    let service = {
        let mut model = locked(shared);
        if !model.begin_check() {
            return;
        }
        model.update_service()
    };
    rebuild();
    let shared = Arc::clone(shared);
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| service.check()))
            .unwrap_or_else(|_| Err(words::JOB_PANICKED.to_owned()));
        locked(&shared).finish_check(result);
        rebuild();
    });
}
