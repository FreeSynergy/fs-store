//! Pattern: Adapter (the top bar and the filter row in the Craft look). The bar: the name, a pill, then on the right the
//! layout switch and refresh in cells with a line between them. The filter row: joined tabs with counts, the sort tabs, the search
//! field, "Update all". The search field is made once and kept, so typing is not lost when the window draws again.

use std::sync::Arc;

use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_components_primitives::SearchField;
use fs_components_styles::styles::craft_sheet::caps;
use fs_components_widgets::Glyph;

use crate::model::{Layout, Sort, StoreModel, Tab};
use crate::view::job::start_refresh;
use crate::view::shell::{Shared, locked, rebuild};
use crate::words;
use crate::words_front as w;

use super::widgets::{Act, Look, button, cell_button, label_dim, tab, tabs};

fn act(shared: &Shared, f: impl Fn(&mut StoreModel) + Send + Sync + 'static) -> Act {
    let shared = Arc::clone(shared);
    Arc::new(move || {
        f(&mut locked(&shared));
        rebuild();
    })
}

fn tab_word(tab: Tab) -> &'static str {
    match tab {
        Tab::All => w::TAB_ALL,
        Tab::Installed => w::TAB_INSTALLED,
        Tab::Updates => w::TAB_UPDATES,
        Tab::NotInstalled => w::TAB_NOT_INSTALLED,
    }
}

fn sort_word(sort: Sort) -> &'static str {
    match sort {
        Sort::Name => w::SORT_NAME,
        Sort::Newest => w::SORT_NEWEST,
        Sort::Size => w::SORT_SIZE,
    }
}

/// A cell of the bar: content with a line on its left.
fn cell(child: Div) -> Div {
    div()
        .class("craft-cell")
        .flex_row()
        .items_center()
        .px(3.5)
        .child(child)
}

/// "Update all (n)" in the bar, when updates wait: a white button, the question comes first.
fn update_all(model: &StoreModel, shared: &Shared) -> Div {
    let waiting = model.updates_waiting();
    let mut slot = div().px(3.0);
    if waiting > 0 {
        let busy = model.is_removing() || model.is_refreshing();
        let shared = Arc::clone(shared);
        slot = slot.child(button(
            "update-all",
            &words::fill(words::BUTTON_UPDATE_ALL, &[("count", &waiting.to_string())]),
            Look::Fill,
            !busy,
            Arc::new(move || {
                locked(&shared).ask_update_all();
                rebuild();
            }),
        ));
    }
    slot
}

/// Height of the top bar.
pub(in crate::view) const BAR_H: f32 = 47.0;

/// The one top line: the name, the program's menus in their own cell, then on the right "Update all", the layout switch and refresh;
/// 47 px high with a line below. The menus come from the shell, so the keyboard reaches them here (Tab, arrows, Enter, Escape).
pub(in crate::view) fn top_bar(model: &StoreModel, shared: &Shared, menus: Option<Div>) -> Div {
    let layout = model.front().layout();
    let refresh = Arc::clone(shared);
    let busy = model.is_removing() || model.is_refreshing();
    let mut toggle = div().flex_row().items_center().gap(0.5);
    for (which, word) in [(Layout::Grid, w::VIEW_GRID), (Layout::Table, w::VIEW_TABLE)] {
        let on = layout == which;
        let mut item = div()
            .id(format!("layout-{word}"))
            .class("craft-pill")
            .attr("tabindex", "0");
        item = if on {
            item.child(text(caps(word)).class("craft-pill-text").no_wrap())
        } else {
            item.class("craft-pill-off").child(label_dim(word))
        };
        toggle = toggle.child(item.on_click({
            let shared = Arc::clone(shared);
            move |_| {
                locked(&shared).set_layout(which);
                rebuild();
            }
        }));
    }
    let mut bar = div()
        .class("craft-bar")
        .flex_row()
        .items_center()
        .h(BAR_H)
        .child(
            div().px(5.0).child(
                text(caps(words::TITLE))
                    .class("craft-caps")
                    .size(14.0)
                    .no_wrap(),
            ),
        );
    if let Some(menus) = menus {
        bar = bar.child(cell(menus));
    }
    bar.child(div().flex_grow())
        .child(update_all(model, shared))
        .child(cell(toggle))
        .child(div().class("craft-cell").child(cell_button(
            "refresh",
            Glyph::Reload,
            if model.is_refreshing() {
                words::REFRESHING
            } else {
                words::BUTTON_REFRESH
            },
            !busy,
            Arc::new(move || start_refresh(&refresh)),
        )))
}

/// The second row: tabs with counts, sort, search, "Update all".
pub(super) fn filter_bar(model: &StoreModel, shared: &Shared, search: &SearchField) -> Div {
    let counts = model.tab_counts();
    let items: Vec<Div> = counts
        .iter()
        .map(|(t, count)| {
            let t = *t;
            tab(
                &format!("tab-{t:?}"),
                tab_word(t),
                Some(*count),
                model.front().tab() == t,
                act(shared, move |m| m.set_tab(t)),
            )
        })
        .collect();
    let sorts: Vec<Div> = Sort::ALL
        .iter()
        .map(|s| {
            let s = *s;
            tab(
                &format!("sort-{s:?}"),
                sort_word(s),
                None,
                model.front().sort() == s,
                act(shared, move |m| m.set_sort(s)),
            )
        })
        .collect();
    div()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(3.0)
        .child(tabs(items))
        .child(div().flex_grow())
        .child(
            div()
                .flex_row()
                .items_center()
                .gap(3.0)
                .child(label_dim(w::SORT_LABEL))
                .child(tabs(sorts)),
        )
        .child(div().w(210.0).flex_shrink_0().child(search.build()))
}
