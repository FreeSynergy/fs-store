//! Pattern: Adapter (the small parts of the storefront: an icon button with its words as a tooltip, a text tab, a badge, a
//! fact row). Every clickable part is a tab stop that Enter and Space press, like the buttons of the other windows.

use std::sync::Arc;

use fs_components::blinc::cn::components::tooltip::TooltipBuilder;
use fs_components::blinc::layout::InstanceKey;
use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_components_widgets::{Glyph, Tone, icon};

/// What a press does.
pub(in crate::view) type Act = Arc<dyn Fn() + Send + Sync>;

/// Side of an icon button.
pub(in crate::view) const ICON_BUTTON: f32 = 36.0;

/// A button that is only an icon; its words are the tooltip. `on` shows it pressed, `enabled` false dims it and does nothing.
pub(in crate::view) fn icon_button(
    id: &str,
    glyph: Glyph,
    tip: &str,
    (on, enabled): (bool, bool),
    act: Act,
) -> Div {
    let key = InstanceKey::explicit(format!("fs-store-tip-{id}"));
    let trigger = move || {
        let mut face = div()
            .class("fs-store-icon-btn")
            .flex_row()
            .items_center()
            .justify_center()
            .w(ICON_BUTTON)
            .h(ICON_BUTTON)
            .child(div().pointer_events_none().child(icon(
                glyph,
                20.0,
                if enabled { Tone::Normal } else { Tone::Muted },
            )));
        if on {
            face = face.class("fs-store-icon-btn-on");
        }
        if !enabled {
            face = face.class("fs-store-icon-btn-off");
        }
        face
    };
    div()
        .id(id.to_owned())
        .flex_shrink_0()
        .w(ICON_BUTTON)
        .h(ICON_BUTTON)
        .attr("tabindex", "0")
        .on_click(move |_| {
            if enabled {
                act();
            }
        })
        .child(TooltipBuilder::with_key(trigger, key).text(tip.to_owned()))
}

/// A text tab or chip: `on` marks the one in force. A locked one (`enabled` false) shows why as its words only.
pub(in crate::view) fn chip(id: &str, label: &str, (on, enabled): (bool, bool), act: Act) -> Div {
    let mut chip = div()
        .id(id.to_owned())
        .class("fs-store-chip")
        .flex_shrink_0()
        .attr("tabindex", "0")
        .child(text(label.to_owned()).no_cursor().no_wrap());
    if on {
        chip = chip.class("fs-store-chip-on");
    }
    if !enabled {
        chip = chip.class("fs-store-chip-off");
    }
    chip.on_click(move |_| {
        if enabled {
            act();
        }
    })
}

/// A small word in a box (a state, a signature level), `tone` picks the colour class.
pub(in crate::view) fn badge(label: &str, tone: &str) -> Div {
    div()
        .class("fs-store-badge")
        .class(tone)
        .flex_shrink_0()
        .child(text(label.to_owned()).no_wrap())
}

/// A label and its value, one line.
pub(in crate::view) fn fact(label: &str, value: &str) -> Div {
    div()
        .class("fs-store-fact")
        .flex_row()
        .gap(3.0)
        .child(
            div()
                .w(110.0)
                .flex_shrink_0()
                .child(text(label.to_owned()).class("fs-store-sub")),
        )
        .child(div().flex_grow().min_w(0.0).child(text(value.to_owned())))
}

/// A heading of a block.
pub(in crate::view) fn heading(label: &str) -> Div {
    div()
        .class("fs-store-heading")
        .child(text(label.to_owned()).bold())
}
