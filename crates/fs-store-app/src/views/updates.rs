// views/updates.rs — List of packages with available updates.

use dioxus::prelude::*;

use crate::context::StoreSignal;
use crate::view::PackageView;

#[component]
pub fn UpdatesList(ctx: StoreSignal) -> Element {
    let state = ctx.read();
    let rows: Vec<_> = state.with_updates().cloned().collect();
    drop(state);

    rsx! {
        div { class: "updates-list",
            h2 { class: "updates-list__title", "Available Updates" }
            if rows.is_empty() {
                p { class: "updates-list__empty", "All packages are up to date." }
            } else {
                for row in &rows {
                    { row.list_item(ctx) }
                }
                p { class: "updates-list__count", "{rows.len()} update(s) available" }
            }
        }
    }
}
