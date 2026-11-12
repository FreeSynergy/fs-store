//! Pattern: Adapter (the list of packages as a grid of cards or as a table, and the start page of blocks).

use std::sync::Arc;

use fs_components::blinc::cn::components::button::ButtonVariant;
use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;

use crate::launcher::Launcher;
use crate::model::{
    Card, CardState, Layout, NeedsState, Section, StartBlock, StoreModel, TableRow, columns,
};
use crate::view::card::{CARD_W, card, logo_small};
use crate::view::shell::{Shared, locked, rebuild};
use crate::words;
use crate::words_front as w;

use super::widgets::badge;

/// Space between cards.
const GAP: f32 = 12.0;

fn grid(
    cards: &[Card],
    model: &StoreModel,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
    room: f32,
) -> Div {
    let mut grid = div().flex_col().gap(GAP / 2.0);
    for chunk in cards.chunks(columns(room, CARD_W, GAP / 2.0)) {
        let mut row = div().flex_row().gap(GAP / 2.0);
        for one in chunk {
            row = row.child(card(one, model.is_removing(), shared, launcher));
        }
        grid = grid.child(row);
    }
    grid
}

fn needs_tone(state: NeedsState) -> &'static str {
    match state {
        NeedsState::None | NeedsState::Met => "fs-store-badge-ok",
        NeedsState::Choice => "fs-store-badge-warn",
        NeedsState::Missing => "fs-store-badge-bad",
    }
}

/// Width of the column of buttons.
const ACTION_W: f32 = 100.0;

const COLS: [(&str, f32); 6] = [
    (w::COL_INSTALLED, 76.0),
    (w::COL_LATEST, 76.0),
    (w::COL_RELEASED, 96.0),
    (w::COL_SIZE, 72.0),
    (w::COL_SIGNATURE, 124.0),
    (w::COL_CAPABILITIES, 70.0),
];

fn cell(width: f32, child: Div) -> Div {
    div().w(width).flex_shrink_0().child(child)
}

fn head() -> Div {
    let mut row = div()
        .class("fs-store-thead")
        .flex_row()
        .items_center()
        .gap(2.0)
        .child(
            div()
                .flex_grow()
                .min_w(0.0)
                .child(text(w::COL_PACKAGE).class("fs-store-sub")),
        );
    for (label, width) in COLS {
        row = row.child(cell(
            width,
            div().child(text(label).class("fs-store-sub").no_wrap()),
        ));
    }
    row.child(div().w(ACTION_W).flex_shrink_0())
}

fn action(row: &TableRow, busy: bool, shared: &Shared, launcher: &Arc<dyn Launcher>) -> Div {
    let name = row.card.name.clone();
    let (label, variant, id) = match row.card.state {
        CardState::NotInstalled => (words::BUTTON_INSTALL, ButtonVariant::Primary, "install"),
        CardState::UpdateAvailable => (words::BUTTON_UPDATE, ButtonVariant::Primary, "update"),
        CardState::Installed => (words::BUTTON_OPEN, ButtonVariant::Secondary, "open"),
    };
    let (shared, launcher) = (Arc::clone(shared), Arc::clone(launcher));
    let state = row.card.state;
    div().w(ACTION_W).flex_shrink_0().child(
        crate::view::shell::button(
            &format!("{id}-{name}"),
            label,
            variant,
            !busy || state == CardState::Installed,
        )
        .on_click(move |_| {
            if state == CardState::Installed {
                locked(&shared).open(&name, None, launcher.as_ref());
            } else {
                locked(&shared).ask_install(&name);
            }
            rebuild();
        }),
    )
}

fn table_row(row: &TableRow, busy: bool, shared: &Shared, launcher: &Arc<dyn Launcher>) -> Div {
    let name = row.card.name.clone();
    let open = Arc::clone(shared);
    let who = div()
        .id(format!("row-{name}"))
        .flex_row()
        .items_center()
        .gap(2.0)
        .flex_grow()
        .min_w(0.0)
        .cursor_pointer()
        .attr("tabindex", "0")
        .child(logo_small(&row.card))
        .child(
            div()
                .flex_col()
                .min_w(0.0)
                .overflow_clip()
                .child(text(row.card.title.clone()).bold().no_wrap())
                .child(
                    text(if row.summary.is_empty() {
                        row.kind.clone()
                    } else {
                        format!("{} - {}", row.kind, row.summary)
                    })
                    .class("fs-store-sub")
                    .no_wrap(),
                ),
        )
        .on_click(move |_| {
            locked(&open).open_page(&name);
            rebuild();
        });
    let plain = |t: &str| div().child(text(t.to_owned()).no_wrap());
    div()
        .class("fs-store-trow")
        .flex_row()
        .items_center()
        .gap(2.0)
        .child(who)
        .child(cell(COLS[0].1, plain(&row.installed)))
        .child(cell(COLS[1].1, plain(&row.latest)))
        .child(cell(COLS[2].1, plain(&row.released)))
        .child(cell(COLS[3].1, plain(&row.size)))
        .child(cell(COLS[4].1, plain(&row.signature)))
        .child(cell(
            COLS[5].1,
            div().child(badge(&row.needs, needs_tone(row.needs_state))),
        ))
        .child(action(row, busy, shared, launcher))
}

fn table(model: &StoreModel, shared: &Shared, launcher: &Arc<dyn Launcher>) -> Div {
    let mut table = div().class("fs-store-table").flex_col().child(head());
    for row in model.table_rows() {
        table = table.child(table_row(&row, model.is_removing(), shared, launcher));
    }
    table
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
        Layout::Grid => grid(&cards, model, shared, launcher, room),
        Layout::Table => table(model, shared, launcher),
    }
}

fn block(
    b: &StartBlock,
    model: &StoreModel,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
    room: f32,
) -> Div {
    let mut top = div()
        .flex_row()
        .items_center()
        .gap(3.0)
        .child(text(b.title.clone()).bold())
        .child(text(b.total.to_string()).class("fs-store-sub"))
        .child(div().flex_grow());
    if let Some(section) = b.more.filter(|_| b.total > b.cards.len()) {
        let shared = Arc::clone(shared);
        top = top.child(
            div()
                .id(format!("more-{}", b.title))
                .class("fs-store-link")
                .attr("tabindex", "0")
                .child(text(words::fill(
                    w::SHOW_ALL,
                    &[("count", &b.total.to_string())],
                )))
                .on_click(move |_| {
                    locked(&shared).open_section(section);
                    rebuild();
                }),
        );
    }
    div()
        .flex_col()
        .gap(GAP / 2.0)
        .child(top)
        .child(grid(&b.cards, model, shared, launcher, room))
}

/// The start page: updates, what is new, the recommended block (locked, with its reason), one block per kind.
pub(super) fn start(
    model: &StoreModel,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
    room: f32,
) -> Div {
    let mut page = div().flex_col().gap(GAP);
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
            .class("fs-store-locked")
            .flex_col()
            .gap(1.0)
            .child(text(w::START_RECOMMENDED).bold())
            .child(text(w::START_RECOMMENDED_WHY).class("fs-store-sub")),
    );
    for b in &blocks {
        page = page.child(block(b, model, shared, launcher, room));
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
