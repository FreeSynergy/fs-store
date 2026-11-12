//! Pattern: Adapter (the list pane of variant C: the search field, the filter chips with their counts, the rows, the count below).

use std::sync::Arc;

use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_components_primitives::SearchField;

use crate::model::{Card, CardState, StoreModel, Tab, ellipsize, summary};
use crate::view::card::logo_plain;
use crate::view::shell::{Shared, locked, rebuild};
use crate::words_front as w;

const SEARCH_H: f32 = 48.0;
const CHIPS_H: f32 = 36.0;
const COUNT_H: f32 = 26.0;
const ROW_LOGO: f32 = 36.0;

fn tab_word(tab: Tab) -> &'static str {
    match tab {
        Tab::All => w::TAB_ALL,
        Tab::Installed => w::TAB_INSTALLED,
        Tab::Updates => w::TAB_UPDATES,
        Tab::NotInstalled => w::TAB_AVAILABLE,
    }
}

fn chips(model: &StoreModel, shared: &Shared) -> Div {
    let mut row = div()
        .class("fs-c-chips")
        .flex_row()
        .flex_wrap()
        .gap(1.0)
        .h(CHIPS_H);
    for (tab, count) in model.tab_counts() {
        let on = model.front().tab() == tab;
        let shared = Arc::clone(shared);
        let mut chip = div()
            .id(format!("c-tab-{tab:?}"))
            .class("fs-c-chip")
            .attr("tabindex", "0");
        let label = format!("{} {count}", tab_word(tab));
        chip = if on {
            chip.class("fs-c-chip-on")
                .child(text(label).class("fs-c-chip-on-text").no_wrap())
        } else {
            chip.child(text(label).no_wrap())
        };
        row = row.child(chip.on_click(move |_| {
            locked(&shared).set_tab(tab);
            rebuild();
        }));
    }
    row
}

/// The mark at the right of a row: what the package needs from the person, in a word.
fn mark(card: &Card, selected: bool) -> Div {
    let (word, class) = match card.state {
        CardState::UpdateAvailable => ("Update", "fs-c-warn"),
        CardState::Installed => ("Installed", "fs-c-ok"),
        CardState::NotInstalled => ("", ""),
    };
    let version = card
        .offered
        .or(card.installed)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let mut col = div().flex_col().items_end().flex_shrink_0();
    let (v, m) = if selected {
        (
            text(version).class("fs-c-on-text").size(12.0).no_wrap(),
            text(word.to_owned())
                .class("fs-c-on-text")
                .size(12.0)
                .no_wrap(),
        )
    } else {
        (
            text(version).class("fs-c-sub").no_wrap(),
            text(word.to_owned()).class(class).size(12.0).no_wrap(),
        )
    };
    col = col.child(v);
    if !word.is_empty() {
        col = col.child(m);
    }
    col
}

fn row(card: &Card, selected: bool, width: f32, shared: &Shared) -> Div {
    let name = card.name.clone();
    let room = (width - 20.0 - ROW_LOGO - 12.0 - 72.0).max(60.0);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let chars = (room / 6.6) as usize;
    let sub = summary(&card.description, chars);
    let sub = if sub.is_empty() {
        ellipsize(w::UNKNOWN, chars)
    } else {
        sub
    };
    let (title, sub_text) = if selected {
        (
            text(card.title.clone())
                .bold()
                .class("fs-c-on-text")
                .no_wrap(),
            text(sub).class("fs-c-on-text").size(12.0).no_wrap(),
        )
    } else {
        (
            text(card.title.clone()).bold().no_wrap(),
            text(sub).class("fs-c-sub").no_wrap(),
        )
    };
    let shared = Arc::clone(shared);
    let mut r = div()
        .id(format!("c-row-{name}"))
        .class("fs-c-row")
        .flex_row()
        .items_center()
        .gap(3.0)
        .attr("tabindex", "0")
        .child(logo_plain(card, ROW_LOGO))
        .child(
            div()
                .flex_col()
                .flex_grow()
                .min_w(0.0)
                .overflow_clip()
                .child(title)
                .child(sub_text),
        )
        .child(mark(card, selected));
    if selected {
        r = r.class("fs-c-row-on");
    }
    r.on_click(move |_| {
        locked(&shared).open_page(&name);
        rebuild();
    })
}

/// The list pane, `width` x `height`.
pub(super) fn pane(
    model: &StoreModel,
    shared: &Shared,
    search: &SearchField,
    (width, height): (f32, f32),
) -> Div {
    let selected = model.page_name().map(str::to_owned);
    let cards = model.listing();
    let mut rows = div().class("fs-c-rows").flex_col().w(width).h((height
        - SEARCH_H
        - CHIPS_H
        - COUNT_H)
        .max(0.0));
    if cards.is_empty() {
        rows = rows.child(
            div()
                .p(3.0)
                .child(text(w::NOTHING_MATCHES).class("fs-c-sub")),
        );
    }
    for card in &cards {
        rows = rows.child(row(
            card,
            selected.as_deref() == Some(card.name.as_str()),
            width,
            shared,
        ));
    }
    div()
        .class("fs-c-list")
        .flex_col()
        .flex_shrink_0()
        .w(width)
        .h(height)
        .child(div().class("fs-c-search").h(SEARCH_H).child(search.build()))
        .child(chips(model, shared))
        .child(rows)
        .child(
            div()
                .class("fs-c-count")
                .flex_row()
                .h(COUNT_H)
                .items_center()
                .gap(3.0)
                .child(text(model.footer_count()))
                .child(div().flex_grow())
                .child(text(model.footer_checked())),
        )
}
