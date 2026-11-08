//! Pattern: Adapter (the one page: a grid of cards, and beside it the description of the selected card).

use std::sync::Arc;

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::launcher::Launcher;
use crate::model::{Card, StoreModel, columns};
use crate::words;

use super::card::{CARD_W, card};
use super::shell::{Shared, button, locked, rebuild};

/// Space between cards and around the page, in pixels.
const GAP: f32 = 12.0;
/// Width of the description panel.
const PANEL_W: f32 = 300.0;
/// What the page keeps for its padding and the scroll bar.
const MARGIN: f32 = 48.0;

/// The description panel of the selected card.
fn panel(selected: &Card, model: &StoreModel, shared: &Shared) -> Div {
    let close = Arc::clone(shared);
    let mut panel = div()
        .class("fs-panel")
        .w(PANEL_W)
        .flex_shrink_0()
        .flex_col()
        .gap(2.0)
        .child(text(selected.name.clone()).size(20.0).bold())
        .child(text(selected.version_line()).class("fs-store-sub"))
        .child(text(selected.description_text()));
    let lines = model.detail_lines(&selected.name);
    if !lines.is_empty() {
        panel = panel.child(text(words::DETAILS_HEADING).bold());
        for line in lines {
            panel = panel.child(text(line).class("fs-store-sub"));
        }
    }
    panel.child(
        button(
            "close-description",
            words::BUTTON_CLOSE,
            ButtonVariant::Outline,
            true,
        )
        .on_click(move |_| {
            locked(&close).close_description();
            rebuild();
        }),
    )
}

/// The page of cards for a window `width` pixels wide.
pub(super) fn cards(
    model: &StoreModel,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
    width: f32,
) -> Div {
    let refresh = Arc::clone(shared);
    let top = div().flex_row().child(
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
    let all = model.cards();
    if all.is_empty() {
        return div().flex_col().w_full().child(top).child(
            div()
                .class("fs-store-empty")
                .child(text(words::EMPTY_CARDS)),
        );
    }
    let selected = model.selected();
    let room = width - MARGIN - selected.as_ref().map_or(0.0, |_| PANEL_W + GAP);
    let mut grid = div().flex_col().gap(GAP / 4.0);
    for chunk in all.chunks(columns(room, CARD_W, GAP)) {
        let mut row = div().flex_row().gap(GAP / 4.0);
        for one in chunk {
            row = row.child(card(one, model.is_busy(), shared, launcher));
        }
        grid = grid.child(row);
    }
    let mut body = div().flex_row().gap(GAP / 4.0).child(grid);
    if let Some(selected) = &selected {
        body = body.child(panel(selected, model, shared));
    }
    div()
        .flex_col()
        .w_full()
        .gap(GAP / 4.0)
        .child(top)
        .child(body)
}
