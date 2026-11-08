//! Pattern: Adapter (one block per installed program: version, where it is, and the buttons Open / Settings).

use std::sync::Arc;

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::launcher::Launcher;
use crate::model::StoreModel;
use crate::words;

use super::shell::{Shared, button, locked, rebuild};

pub(super) fn installed(model: &StoreModel, shared: &Shared, launcher: &Arc<dyn Launcher>) -> Div {
    let mut page = div().flex_col().w_full();
    if model.installed().is_empty() {
        return page.child(
            div()
                .class("fs-store-empty")
                .child(text(words::EMPTY_INSTALLED)),
        );
    }
    for row in model.installed() {
        let mut cell = div()
            .class("fs-store-cell")
            .flex_col()
            .child(text(format!("{} {}", row.name, row.version)));
        for line in model.detail_lines(&row.name) {
            cell = cell.child(text(line).class("fs-store-sub"));
        }
        let mut buttons = div().flex_row().child(open_button(
            &row.name,
            None,
            words::BUTTON_OPEN,
            shared,
            launcher,
        ));
        for facet in model.details(&row.name).facets {
            let label = format!("{} {facet}", words::BUTTON_OPEN);
            buttons = buttons.child(open_button(
                &row.name,
                Some(facet),
                &label,
                shared,
                launcher,
            ));
        }
        buttons = buttons
            .child(button(
                &format!("settings-{}", row.name),
                words::BUTTON_PROGRAM_SETTINGS,
                ButtonVariant::Ghost,
                false,
            ))
            .child(text(words::SETTINGS_LATER).class("fs-store-sub"));
        page = page.child(div().class("fs-store-row").child(cell).child(buttons));
    }
    page
}

fn open_button(
    name: &str,
    facet: Option<String>,
    label: &str,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
) -> impl blinc_layout::ElementBuilder + use<> {
    let (shared, launcher, package) = (Arc::clone(shared), Arc::clone(launcher), name.to_owned());
    let key = format!("open-{name}-{}", facet.as_deref().unwrap_or(""));
    button(&key, label, ButtonVariant::Secondary, true).on_click(move |_| {
        locked(&shared).open(&package, facet.as_deref(), launcher.as_ref());
        rebuild();
    })
}
