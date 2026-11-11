//! Pattern: Adapter (the sources section of the admin page: a field and buttons that call the model's commands).

use std::sync::Arc;

use fs_components::blinc::cn::components::button::ButtonVariant;
use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_components_primitives::SearchField;

use crate::model::StoreModel;
use crate::words;

use super::job::read_if_wanted;
use super::shell::{Shared, button, locked, rebuild};

pub(super) fn sources_section(model: &StoreModel, shared: &Shared) -> Div {
    let typed = Arc::clone(shared);
    let field = SearchField::new(words::ADD_PLACEHOLDER).on_change(Arc::new(move |t: &str| {
        locked(&typed).set_new_source(t);
    }));
    let add = Arc::clone(shared);
    let mut page = div().flex_col().w_full().child(
        div().class("fs-store-add").child(field.build()).child(
            button(
                "add-source",
                words::BUTTON_ADD,
                ButtonVariant::Primary,
                true,
            )
            .on_click(move |_| {
                locked(&add).add_source();
                read_if_wanted(&add);
                rebuild();
            }),
        ),
    );
    if model.sources().is_empty() {
        return page.child(
            div()
                .class("fs-store-empty")
                .child(text(words::EMPTY_SOURCES)),
        );
    }
    for source in model.sources() {
        let mut name = source.id.clone();
        if source.reference {
            name = format!("{name} ({})", words::MARK_BUILT_IN);
        }
        if !source.enabled {
            name = format!("{name} ({})", words::MARK_OFF);
        }
        let (toggle_label, toggle_variant) = if source.enabled {
            (words::BUTTON_SWITCH_OFF, ButtonVariant::Outline)
        } else {
            (words::BUTTON_SWITCH_ON, ButtonVariant::Secondary)
        };
        let (toggle, remove) = (Arc::clone(shared), Arc::clone(shared));
        let (toggle_id, remove_id, now) = (source.id.clone(), source.id.clone(), source.enabled);
        let mut row = div()
            .class("fs-store-row")
            .child(
                div()
                    .class("fs-store-cell")
                    .flex_col()
                    .child(text(name))
                    .child(text(source.url.clone()).class("fs-store-sub")),
            )
            .child(
                button(
                    &format!("toggle-{}", source.id),
                    toggle_label,
                    toggle_variant,
                    true,
                )
                .on_click(move |_| {
                    locked(&toggle).set_enabled(&toggle_id, !now);
                    read_if_wanted(&toggle);
                    rebuild();
                }),
            );
        // The reference source cannot be removed: no button for it (the model refuses anyway).
        if !source.reference {
            row = row.child(
                button(
                    &format!("remove-{}", source.id),
                    words::BUTTON_REMOVE,
                    ButtonVariant::Destructive,
                    true,
                )
                .on_click(move |_| {
                    locked(&remove).remove_source(&remove_id);
                    read_if_wanted(&remove);
                    rebuild();
                }),
            );
        }
        page = page.child(row);
    }
    page
}
