//! Pattern: Adapter (the update choices of the admin page: choices as rows of buttons, the chosen one stands out; each click
//! calls a model command that saves).

use std::sync::Arc;

use fs_components::blinc::cn::components::button::ButtonVariant;
use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_store_sources::{CheckEvery, InstallMode};

use crate::model::{CheckState, StoreModel, check_label, install_label};
use crate::words;

use super::check::start_check;
use super::shell::{Shared, button, locked, rebuild};

/// One choice: a button, highlighted when it is the chosen one.
pub(super) fn choice(
    key: &str,
    label: &str,
    chosen: bool,
    on: impl Fn() + Send + Sync + 'static,
) -> Div {
    let variant = if chosen {
        ButtonVariant::Secondary
    } else {
        ButtonVariant::Outline
    };
    div().child(button(key, label, variant, true).on_click(move |_| {
        on();
        rebuild();
    }))
}

pub(super) fn section(heading: &str, choices: Div) -> Div {
    div()
        .flex_col()
        .class("fs-store-section")
        .child(text(heading).bold())
        .child(choices)
}

pub(super) fn updates_section(model: &StoreModel, shared: &Shared) -> Div {
    let now = model.settings();
    let mut every = div().class("fs-store-add");
    for value in [
        CheckEvery::Off,
        CheckEvery::Start,
        CheckEvery::Daily,
        CheckEvery::Hourly,
    ] {
        let shared = Arc::clone(shared);
        every = every.child(choice(
            &format!("check-{value:?}"),
            check_label(value),
            now.check == value,
            move || locked(&shared).set_check_every(value),
        ));
    }
    let mut install = div().class("fs-store-add");
    for value in [InstallMode::Ask, InstallMode::Auto] {
        let shared = Arc::clone(shared);
        install = install.child(choice(
            &format!("install-{value:?}"),
            install_label(value),
            now.install == value,
            move || locked(&shared).set_install_mode(value),
        ));
    }
    let mut notify = div().class("fs-store-add");
    for (value, label) in [(true, words::NOTIFY_ON), (false, words::NOTIFY_OFF)] {
        let shared = Arc::clone(shared);
        notify = notify.child(choice(
            &format!("notify-{value}"),
            label,
            now.notify == value,
            move || locked(&shared).set_notify(value),
        ));
    }
    let running = *model.check_state() == CheckState::Running;
    let look = Arc::clone(shared);
    let mut now_row = div()
        .class("fs-store-add")
        .child(text(model.last_check_line()))
        .child(
            button(
                "look-now",
                words::BUTTON_LOOK_NOW,
                ButtonVariant::Primary,
                !running,
            )
            .on_click(move |_| start_check(&look)),
        );
    if let Some(line) = model.check_line() {
        now_row = now_row.child(text(line).class("fs-store-sub"));
    }
    div()
        .flex_col()
        .w_full()
        .child(section(words::SETTINGS_CHECK, every))
        .child(
            section(words::SETTINGS_INSTALL, install)
                .child(text(words::INSTALL_AUTO_HINT).class("fs-store-sub")),
        )
        .child(section(words::SETTINGS_NOTIFY, notify))
        .child(div().class("fs-store-section").child(now_row))
}
