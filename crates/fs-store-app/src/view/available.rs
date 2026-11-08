//! Pattern: Adapter (the available screen: one row per offer, with the button the model's action names).

use std::sync::Arc;

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::model::{Action, AvailableRow, StoreModel};
use crate::words;

use super::job::start_job;
use super::shell::{Shared, button, locked, rebuild};

pub(super) fn available(model: &StoreModel, shared: &Shared) -> Div {
    let refresh = Arc::clone(shared);
    let mut page = div().flex_col().w_full().child(
        button(
            "refresh",
            words::BUTTON_REFRESH,
            ButtonVariant::Ghost,
            !model.is_busy(),
        )
        .on_click(move |_| {
            locked(&refresh).refresh();
            rebuild();
        }),
    );
    if model.available().is_empty() {
        return page.child(
            div()
                .class("fs-store-empty")
                .child(text(words::EMPTY_AVAILABLE)),
        );
    }
    for row in model.available() {
        page = page.child(available_row(
            row,
            model.is_busy(),
            model.is_fresh(&row.package.name),
            shared,
        ));
    }
    page
}

fn available_row(row: &AvailableRow, busy: bool, fresh: bool, shared: &Shared) -> Div {
    let package = &row.package;
    let version = package.version.to_string();
    let sub = match row.installed {
        Some(installed) => words::fill(
            words::NEWER_OFFERED,
            &[
                ("offered", version.as_str()),
                ("installed", installed.to_string().as_str()),
            ],
        ),
        None => version.clone(),
    };
    let name_line = if fresh {
        format!("{} ({})", package.name, words::MARK_NEW)
    } else {
        package.name.clone()
    };
    let line = format!("{} - {}", words::kind(package.kind), sub);
    let cell = div()
        .class("fs-store-cell")
        .flex_col()
        .child(text(name_line))
        .child(text(line).class("fs-store-sub"));
    let action: Div = match row.action {
        Action::UpToDate => div().child(
            text(words::fill(
                words::UP_TO_DATE,
                &[("version", version.as_str())],
            ))
            .class("fs-store-sub"),
        ),
        Action::Install | Action::Update => {
            let label = if row.action == Action::Install {
                words::BUTTON_INSTALL
            } else {
                words::BUTTON_UPDATE
            };
            let (shared, name) = (Arc::clone(shared), package.name.clone());
            div().child(
                button(
                    &format!("job-{}", package.name),
                    label,
                    ButtonVariant::Primary,
                    !busy,
                )
                .on_click(move |_| start_job(&shared, &name)),
            )
        }
    };
    div().class("fs-store-row").child(cell).child(action)
}
