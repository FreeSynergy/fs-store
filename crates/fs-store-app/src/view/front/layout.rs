//! Pattern: Facade (the storefront as one element tree: the bar, the navigation, the main area with the jobs, the problems, the
//! question before any work, the page or the list, and the footer). The search field is made once and kept.

use std::sync::Arc;

use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_components_primitives::SearchField;

use crate::launcher::Launcher;
use crate::model::StoreModel;
use crate::words_front as w;

use super::super::shell::{Shared, locked, rebuild};
use super::widgets::label_dim;
use super::{bar, list, nav, page, preview};

/// How long the window waits after the last key before it narrows the list: a key changes only the text of the field
/// (fast); the list is drawn again once the typing pauses, not once per key.
const SEARCH_PAUSE: std::time::Duration = std::time::Duration::from_millis(180);

/// The search field: made once, kept for the life of the window, so a redraw keeps what is typed.
pub(in crate::view) fn search_field(shared: &Shared) -> SearchField {
    let shared = Arc::clone(shared);
    let latest = Arc::new(std::sync::atomic::AtomicU64::new(0));
    SearchField::new(w::SEARCH_PLACEHOLDER).on_change(Arc::new(move |t: &str| {
        use std::sync::atomic::Ordering;
        locked(&shared).set_search(t);
        let mine = latest.fetch_add(1, Ordering::SeqCst) + 1;
        let latest = Arc::clone(&latest);
        std::thread::spawn(move || {
            std::thread::sleep(SEARCH_PAUSE);
            if latest.load(Ordering::SeqCst) == mine {
                rebuild();
            }
        });
    }))
}

/// The footer: when the last look for updates was and where the packages come from (left), how many and in what form they are
/// installed (right); 33 px high with a line above.
fn footer(model: &StoreModel) -> Div {
    div()
        .class("craft-footer")
        .flex_row()
        .items_center()
        .gap(4.0)
        .child(label_dim(&model.footer_checked()))
        .child(
            div()
                .flex_shrink()
                .min_w(0.0)
                .overflow_clip()
                .child(label_dim(&model.footer_sources())),
        )
        .child(div().flex_grow())
        .child(label_dim(&model.footer_count()))
        .child(label_dim(&model.footer_format()))
}

/// The whole storefront for a window `width` x `height`.
pub(in crate::view) fn storefront(
    model: &StoreModel,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
    search: &SearchField,
    (width, height): (f32, f32),
) -> Div {
    let room = width - nav::NAV_W - 48.0 - 1.0;
    let mut main = div()
        .class("fs-store-main")
        .flex_col()
        .gap(4.0)
        .min_w(0.0)
        .flex_grow();
    if !model.jobs().is_empty() {
        main = main.child(crate::view::job::job_list(model, shared));
    }
    for line in model.problem_lines() {
        main = main.child(div().class("fs-store-problem").child(text(line)));
    }
    if let Some(p) = preview::preview(model, shared) {
        main = main.child(p);
    }
    if let Some(name) = model.page_name() {
        main = main.child(page::page(model, shared, launcher, name));
    } else {
        let body = if list::shows_list(model) {
            list::listing(model, shared, launcher, room)
        } else {
            list::start(model, shared, launcher, room)
        };
        main = main
            .child(bar::filter_bar(model, shared, search))
            .child(body);
    }
    div()
        .flex_col()
        .w(width)
        .h(height)
        .child(
            div()
                .class("fs-store-body")
                .flex_row()
                .flex_grow()
                .min_h(0.0)
                .child(nav::nav(model, shared))
                .child(main),
        )
        .child(footer(model))
}
