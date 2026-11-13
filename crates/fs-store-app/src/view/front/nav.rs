//! Pattern: Adapter (the navigation at the left in the Craft look: capital mono labels with their counts; the one shown is white
//! with black capitals; an entry that cannot be chosen is dim and says why, so it stays visible).

use std::sync::Arc;

use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_components_styles::styles::craft_sheet::caps;

use crate::model::{NavEntry, StoreModel};
use crate::view::shell::{Shared, locked, rebuild};

use fs_components_widgets::craft::label_dim;

/// Width of the navigation.
pub(super) const NAV_W: f32 = 190.0;

fn item(entry: &NavEntry, i: usize, shared: &Shared) -> Div {
    let mut line = div()
        .id(format!("nav-{i}"))
        .class("craft-nav-item")
        .flex_col()
        .gap(0.5)
        .attr("tabindex", "0");
    let mut head = div().flex_row().items_center().gap(2.0);
    let count = entry.count.map(|c| c.to_string()).unwrap_or_default();
    if entry.current {
        line = line.class("craft-tab-on");
        head = head
            .child(
                div()
                    .flex_grow()
                    .child(text(caps(entry.label)).class("craft-pill-text").no_wrap()),
            )
            .child(text(count).class("craft-pill-text").no_wrap());
    } else if entry.enabled {
        head = head
            .child(
                div()
                    .flex_grow()
                    .child(text(caps(entry.label)).class("craft-caps").no_wrap()),
            )
            .child(text(count).class("craft-caps-dim").no_wrap());
    } else {
        line = line.class("craft-nav-off");
        head = head.child(div().flex_grow().child(label_dim(entry.label)));
    }
    line = line.child(head);
    if let Some(note) = entry.note {
        line = line.child(text(caps(note)).class("craft-note").no_wrap());
    }
    if let (true, Some(section)) = (entry.enabled, entry.section) {
        let shared = Arc::clone(shared);
        line = line.on_click(move |_| {
            locked(&shared).open_section(section);
            rebuild();
        });
    }
    line
}

pub(super) fn nav(model: &StoreModel, shared: &Shared) -> Div {
    let mut column = div()
        .class("craft-nav")
        .flex_col()
        .flex_shrink_0()
        .w(NAV_W)
        .gap(1.0);
    for (i, entry) in model.nav().iter().enumerate() {
        column = column.child(item(entry, i, shared));
    }
    column
}
