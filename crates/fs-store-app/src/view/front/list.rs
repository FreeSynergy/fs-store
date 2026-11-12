//! Pattern: Adapter (the list of packages as a grid of tiles that share their lines, or as a table, and the start page of numbered
//! sections in the Craft look).

use std::sync::Arc;

use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_components_styles::styles::craft_sheet::caps;

use crate::launcher::Launcher;
use crate::model::{Card, Layout, Section, StartBlock, StoreModel, columns};
use crate::view::shell::{Shared, locked, rebuild};
use crate::words;
use crate::words_front as w;

use super::tile::tile;
use super::widgets::label_dim;

/// The least width of a tile before the grid takes one column fewer.
const TILE_MIN: f32 = 300.0;

/// A grid of tiles: the lines are shared, the row is as wide as the room, the numbers start at `first`.
fn grid(
    cards: &[Card],
    first: usize,
    model: &StoreModel,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
    room: f32,
) -> Div {
    let cols = columns(room, TILE_MIN, 0.0);
    #[allow(clippy::cast_precision_loss)]
    let width = (room / cols as f32).floor();
    let mut grid = div().class("craft-grid").flex_col().flex_shrink_0();
    for (r, chunk) in cards.chunks(cols).enumerate() {
        let mut row = div().flex_row();
        for (i, one) in chunk.iter().enumerate() {
            row = row.child(tile(
                one,
                first + r * cols + i,
                width,
                model.is_removing(),
                shared,
                launcher,
            ));
        }
        grid = grid.child(row);
    }
    grid
}

/// The list of the section as the person chose it, or a sentence when nothing matches.
pub(super) fn listing(
    model: &StoreModel,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
    room: f32,
) -> Div {
    let cards = model.listing();
    if cards.is_empty() {
        return div()
            .class("fs-store-empty")
            .child(text(if model.cards().is_empty() {
                words::EMPTY_CARDS
            } else {
                w::NOTHING_MATCHES
            }));
    }
    match model.front().layout() {
        Layout::Grid => div()
            .flex_col()
            .gap(3.5)
            .child(section_title(1, w::SECTION_ALL, cards.len(), None, shared))
            .child(grid(&cards, 1, model, shared, launcher, room)),
        Layout::Table => super::table::table(model, shared, launcher),
    }
}

/// `01 / PROGRAMS` on the left, the count (or "Show all") on the right.
fn section_title(
    number: usize,
    title: &str,
    total: usize,
    more: Option<Section>,
    shared: &Shared,
) -> Div {
    let mut row = div()
        .flex_row()
        .items_center()
        .child(label_dim(&words::fill(
            w::SECTION_TITLE,
            &[("number", &format!("{number:02}")), ("title", title)],
        )))
        .child(div().flex_grow());
    row = match more {
        Some(section) => {
            let shared = Arc::clone(shared);
            row.child(
                div()
                    .id(format!("more-{title}"))
                    .class("craft-link")
                    .attr("tabindex", "0")
                    .child(
                        text(format!(
                            "{} \u{2197}",
                            caps(&words::fill(w::SHOW_ALL, &[("count", &total.to_string())]))
                        ))
                        .class("craft-caps-dim")
                        .no_wrap(),
                    )
                    .on_click(move |_| {
                        locked(&shared).open_section(section);
                        rebuild();
                    }),
            )
        }
        None => row.child(label_dim(&words::fill(
            w::SECTION_COUNT,
            &[("count", &total.to_string())],
        ))),
    };
    row
}

fn block(
    number: usize,
    b: &StartBlock,
    model: &StoreModel,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
    room: f32,
) -> Div {
    div()
        .flex_col()
        .gap(3.5)
        .child(section_title(
            number,
            &b.title,
            b.total,
            b.more.filter(|_| b.total > b.cards.len()),
            shared,
        ))
        .child(grid(&b.cards, 1, model, shared, launcher, room))
}

/// The start page: updates, what is new, the recommended block (locked, with its reason), one section per kind.
pub(super) fn start(
    model: &StoreModel,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
    room: f32,
) -> Div {
    let mut page = div().flex_col().gap(7.0);
    let blocks = model.start_blocks();
    if blocks.is_empty() {
        return page.child(
            div()
                .class("fs-store-empty")
                .child(text(words::EMPTY_CARDS)),
        );
    }
    page = page.child(
        div()
            .class("craft-panel")
            .flex_col()
            .gap(1.0)
            .p(4.0)
            .child(text(caps(w::START_RECOMMENDED)).class("craft-caps"))
            .child(text(w::START_RECOMMENDED_WHY).class("craft-sub")),
    );
    for (i, b) in blocks.iter().enumerate() {
        page = page.child(block(i + 1, b, model, shared, launcher, room));
    }
    page
}

/// The start page shows blocks; a search, another tab, a section or the table layout shows the list.
pub(super) fn shows_list(model: &StoreModel) -> bool {
    let f = model.front();
    f.section() != Section::Start
        || !f.search().trim().is_empty()
        || f.tab() != crate::model::Tab::All
        || f.layout() == Layout::Table
}
