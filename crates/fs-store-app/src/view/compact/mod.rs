//! Pattern: Facade (variant C, compact: the list of packages at the left with the search and the filter, the page of the selected
//! package at the right, one pane next to the other as in the software centres of the desktops). It reads the same model as variant
//! B; only the elements differ. Split by topic: `list` (search, filter, rows), `detail` (the page of one package), `preview` (the
//! question before any work), `styles`.

mod detail;
mod list;
mod preview;
mod styles;

use std::sync::Arc;

use fs_components::blinc::layout::div::{Div, div};
use fs_components_primitives::SearchField;

use crate::launcher::Launcher;
use crate::model::StoreModel;

use super::shell::Shared;

pub(super) use styles::STYLES;

/// Width of the list at the left.
const LIST_W: f32 = 380.0;

/// The whole compact window body for `width` x `height`. The first package of the list is selected when none is, so the right
/// pane is never empty while there is something to show.
pub(super) fn compact(
    model: &mut StoreModel,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
    search: &SearchField,
    (width, height): (f32, f32),
) -> Div {
    let shown = model.listing();
    if model
        .page_name()
        .is_none_or(|n| !shown.iter().any(|c| c.name == n))
        && let Some(first) = shown.first()
    {
        let name = first.name.clone();
        model.open_page(&name);
    }
    div()
        .class("fs-c-root")
        .flex_row()
        .w(width)
        .h(height)
        .child(list::pane(
            model,
            shared,
            search,
            (LIST_W.min(width), height),
        ))
        .child(detail::pane(
            model,
            shared,
            launcher,
            ((width - LIST_W).max(0.0), height),
        ))
}
