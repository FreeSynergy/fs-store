//! Pattern: Adapter (the top bar: layout switch, refresh, update all; below it the filter tabs, the sort choice and the search
//! field). The search field is made once and kept, so typing is not lost when the window draws again.

use std::sync::Arc;

use fs_components::blinc::cn::components::button::ButtonVariant;
use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_components_primitives::SearchField;
use fs_components_widgets::Glyph;

use crate::model::{Layout, Sort, StoreModel, Tab};
use crate::view::job::start_refresh;
use crate::view::shell::{Shared, button, locked, rebuild};
use crate::words;
use crate::words_front as w;

use super::widgets::{Act, chip, icon_button};

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

/// The first row: the name of the window on the left, the layout switch, refresh and "Update all" on the right.
pub(super) fn top_bar(model: &StoreModel, shared: &Shared, search: &SearchField) -> Div {
    let layout = model.front().layout();
    let waiting = model.updates_waiting();
    let busy = model.is_removing() || model.is_refreshing();
    let refresh = Arc::clone(shared);
    let mut bar = div()
        .class("fs-store-top")
        .flex_row()
        .items_center()
        .gap(2.0)
        .child(text(words::TITLE).bold().size(18.0))
        .child(div().w(36.0).flex_shrink_0())
        .child(div().w(320.0).flex_shrink_0().child(search.build()))
        .child(div().flex_grow())
        .child(icon_button(
            "layout-grid",
            Glyph::ViewGrid,
            w::VIEW_GRID,
            (layout == Layout::Grid, true),
            act(shared, |m| m.set_layout(Layout::Grid)),
        ))
        .child(icon_button(
            "layout-table",
            Glyph::ViewList,
            w::VIEW_TABLE,
            (layout == Layout::Table, true),
            act(shared, |m| m.set_layout(Layout::Table)),
        ))
        .child(icon_button(
            "refresh",
            Glyph::Reload,
            if model.is_refreshing() {
                words::REFRESHING
            } else {
                words::BUTTON_REFRESH
            },
            (model.is_refreshing(), !busy),
            Arc::new(move || start_refresh(&refresh)),
        ));
    if waiting > 0 {
        bar = bar.child(
            button(
                "update-all",
                &words::fill(words::BUTTON_UPDATE_ALL, &[("count", &waiting.to_string())]),
                ButtonVariant::Primary,
                !busy,
            )
            .on_click({
                let shared = Arc::clone(shared);
                move |_| {
                    locked(&shared).ask_update_all();
                    rebuild();
                }
            }),
        );
    }
    bar
}

/// The second row: tabs with their counts, the sort choice, the search field.
pub(super) fn filter_bar(model: &StoreModel, shared: &Shared) -> Div {
    let mut tabs = div().flex_row().items_center().gap(1.0);
    for (tab, count) in model.tab_counts() {
        let label = format!("{} {count}", tab_word(tab));
        tabs = tabs.child(chip(
            &format!("tab-{tab:?}"),
            &label,
            (model.front().tab() == tab, true),
            act(shared, move |m| m.set_tab(tab)),
        ));
    }
    let mut sort = div()
        .flex_row()
        .items_center()
        .gap(1.0)
        .child(text(w::SORT_LABEL).class("fs-store-sub"));
    for choice in Sort::ALL {
        sort = sort.child(chip(
            &format!("sort-{choice:?}"),
            sort_word(choice),
            (model.front().sort() == choice, true),
            act(shared, move |m| m.set_sort(choice)),
        ));
    }
    div()
        .class("fs-store-filters")
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(3.0)
        .child(tabs)
        .child(div().flex_grow())
        .child(sort)
}
