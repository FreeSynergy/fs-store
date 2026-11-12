//! Pattern: Adapter (the small parts of the Craft look the storefront is made of: capital mono labels, square buttons, joined
//! tabs, badges, numbered boxes, the progress bar, an icon cell). Every part takes its look from the `craft-*` classes of
//! `fs-components-styles` and the theme variables; nothing is drawn here. Clickable parts are tab stops that Enter and Space press.

use std::sync::Arc;

use fs_components::blinc::cn::components::tooltip::TooltipBuilder;
use fs_components::blinc::layout::InstanceKey;
use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_components_styles::styles::craft_sheet::caps;
use fs_components_widgets::{Glyph, Tone, icon};

/// What a press does.
pub(in crate::view) type Act = Arc<dyn Fn() + Send + Sync>;

/// A small label in capitals, mono, spaced (the sheet gives face, size and spacing).
pub(in crate::view) fn label(
    words: &str,
) -> impl fs_components::blinc::layout::ElementBuilder + use<> {
    text(caps(words)).class("craft-caps").no_wrap()
}

/// The same, dim.
pub(in crate::view) fn label_dim(
    words: &str,
) -> impl fs_components::blinc::layout::ElementBuilder + use<> {
    text(caps(words)).class("craft-caps-dim").no_wrap()
}

/// How a square button looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::view) enum Look {
    /// White, black capitals.
    Fill,
    /// An outline.
    Line,
}

/// A square button: capitals in the mono face, 34 px high. `enabled` false dims it and does nothing (a locked part says why as a tooltip).
pub(in crate::view) fn button(id: &str, words: &str, look: Look, enabled: bool, act: Act) -> Div {
    let mut b = div()
        .id(id.to_owned())
        .class("craft-btn")
        .flex_row()
        .items_center()
        .justify_center()
        .px(3.5)
        .attr("tabindex", "0")
        .child(match look {
            Look::Fill => text(caps(words)).class("craft-btn-fill-text").no_wrap(),
            Look::Line => text(caps(words)).class("craft-caps").no_wrap(),
        });
    b = match look {
        Look::Fill => b.class("craft-btn-fill"),
        Look::Line => b.class("craft-btn-line"),
    };
    if !enabled {
        b = b.class("craft-btn-off");
    }
    b.on_click(move |_| {
        if enabled {
            act();
        }
    })
}

/// A square cell that holds an icon or a few characters (refresh, pause, the three dots), 34 px high.
pub(in crate::view) fn cell_button(
    id: &str,
    glyph: Glyph,
    tip: &str,
    enabled: bool,
    act: Act,
) -> Div {
    let key = InstanceKey::explicit(format!("fs-store-tip-{id}"));
    let tone = if enabled { Tone::Normal } else { Tone::Muted };
    let trigger = move || {
        let mut face = div()
            .class("craft-btn")
            .flex_row()
            .items_center()
            .justify_center()
            .w(36.0)
            .child(div().pointer_events_none().child(icon(glyph, 16.0, tone)));
        if !enabled {
            face = face.class("craft-btn-off");
        }
        face
    };
    div()
        .id(id.to_owned())
        .flex_shrink_0()
        .attr("tabindex", "0")
        .on_click(move |_| {
            if enabled {
                act();
            }
        })
        .child(TooltipBuilder::with_key(trigger, key).text(tip.to_owned()))
}

/// One tab of a joined row of tabs: the one in force is white with black capitals; `count` is shown dim after the words.
pub(in crate::view) fn tab(id: &str, words: &str, count: Option<usize>, on: bool, act: Act) -> Div {
    let mut t = div()
        .id(id.to_owned())
        .class("craft-tab")
        .flex_row()
        .items_center()
        .gap(1.5)
        .attr("tabindex", "0");
    if on {
        t = t
            .class("craft-tab-on")
            .child(text(caps(words)).class("craft-pill-text").no_wrap());
        if let Some(n) = count {
            t = t.child(text(n.to_string()).class("craft-pill-text").no_wrap());
        }
    } else {
        t = t.child(label(words));
        if let Some(n) = count {
            t = t.child(text(n.to_string()).class("craft-caps-dim").no_wrap());
        }
    }
    t.on_click(move |_| act())
}

/// A row of tabs inside one box with a line around it (the last tab has no line to its right).
pub(in crate::view) fn tabs(items: Vec<Div>) -> Div {
    let mut row = div().class("craft-box").flex_row().flex_shrink_0();
    for item in items {
        row = row.child(item);
    }
    row
}

/// A badge: a small box with a 1 px line and a deep ground, the words in capitals in the colour of the state.
/// `tone` is one of `fs-t-ok`, `fs-t-warn`, `fs-t-info`, `fs-t-bad`, `fs-t-quiet`.
pub(in crate::view) fn badge(words: &str, tone: &str) -> Div {
    div()
        .class("craft-badge")
        .flex_shrink_0()
        .child(text(caps(words)).class("craft-caps").class(tone).no_wrap())
}

/// The numbered box of a tile: `number` in a block of the package's accent colour.
pub(in crate::view) fn number_box(number: usize, accent: (u8, u8, u8)) -> Div {
    div()
        .class("craft-number")
        .flex_row()
        .items_center()
        .justify_center()
        .bg(fs_components::blinc::core::Color::rgb(
            f32::from(accent.0) / 255.0,
            f32::from(accent.1) / 255.0,
            f32::from(accent.2) / 255.0,
        ))
        .child(
            text(format!("{number:02}"))
                .class("craft-number-text")
                .no_wrap(),
        )
}

/// A thin bar `share` (0..1) full, `width` pixels wide.
pub(in crate::view) fn progress(share: f32, width: f32, accent: (u8, u8, u8)) -> Div {
    div().class("craft-progress-track").w(width).child(
        div().h(4.0).w((width * share.clamp(0.0, 1.0)).max(1.0)).bg(
            fs_components::blinc::core::Color::rgb(
                f32::from(accent.0) / 255.0,
                f32::from(accent.1) / 255.0,
                f32::from(accent.2) / 255.0,
            ),
        ),
    )
}

/// A label above its value: the label in dim capitals (long labels and long values both fit).
pub(in crate::view) fn fact(name: &str, value: &str) -> Div {
    div()
        .flex_col()
        .gap(1.0)
        .child(label_dim(name))
        .child(text(value.to_owned()))
}

/// A heading of a block: dim capitals.
pub(in crate::view) fn heading(words: &str) -> Div {
    div().class("craft-section").child(label_dim(words))
}
