//! Pattern: Adapter (the navigation at the left: the start page and one entry per kind of package; an entry the catalog
//! cannot fill yet is locked and says why).

use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_components_widgets::{Glyph, Tone, icon};

use crate::model::{NavEntry, Section, StoreModel};
use crate::view::shell::{Shared, locked, rebuild};
use crate::words_front as w;

use fs_store_core::Kind;

fn glyph(entry: &NavEntry) -> Glyph {
    match entry.section {
        Some(Section::Start) => Glyph::Home,
        Some(Section::Kind(Kind::Program)) => Glyph::FileCode,
        Some(Section::Kind(Kind::Frontend)) => Glyph::ViewGrid,
        Some(Section::Kind(Kind::Component)) => Glyph::FileArchive,
        Some(Section::Kind(Kind::Theme)) => Glyph::FileImage,
        None if entry.label == w::NAV_SERVERS => Glyph::Drive,
        None => Glyph::Info,
    }
}

/// Width of the navigation.
pub(super) const NAV_W: f32 = 200.0;

pub(super) fn nav(model: &StoreModel, shared: &Shared) -> Div {
    let mut column = div()
        .class("fs-store-nav")
        .flex_col()
        .flex_shrink_0()
        .w(NAV_W)
        .gap(1.0);
    for (i, entry) in model.nav().into_iter().enumerate() {
        let tone = if entry.enabled {
            Tone::Normal
        } else {
            Tone::Muted
        };
        let mut label = div()
            .flex_col()
            .flex_grow()
            .min_w(0.0)
            .child(text(entry.label).no_wrap());
        if let Some(note) = entry.note {
            label = label.child(text(note).class("fs-store-sub").size(11.0).no_wrap());
        }
        let mut line = div()
            .id(format!("nav-{i}"))
            .class("fs-store-nav-item")
            .flex_row()
            .items_center()
            .gap(2.0)
            .attr("tabindex", "0")
            .child(
                div()
                    .pointer_events_none()
                    .child(icon(glyph(&entry), 18.0, tone)),
            )
            .child(label);
        if entry.enabled {
            line = line.child(text(entry.count.to_string()).class("fs-store-sub"));
        }
        if entry.current {
            line = line.class("fs-store-nav-current");
        }
        if !entry.enabled {
            line = line.class("fs-store-nav-off");
        }
        if let (true, Some(section)) = (entry.enabled, entry.section) {
            let shared = std::sync::Arc::clone(shared);
            line = line.on_click(move |_| {
                locked(&shared).open_section(section);
                rebuild();
            });
        }
        column = column.child(line);
    }
    column
}
