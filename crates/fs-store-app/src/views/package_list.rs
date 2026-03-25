// views/package_list.rs — browsable list of all Store packages.

use dioxus::prelude::*;

use crate::context::StoreSignal;
use crate::view::PackageView;

const NAMESPACES: &[(&str, &str)] = &[
    ("apps", "Apps"),
    ("containers", "Containers"),
    ("themes", "Themes"),
    ("widgets", "Widgets"),
    ("bundles", "Bundles"),
    ("icons", "Icons"),
    ("languages", "Languages"),
    ("externals", "External"),
    ("repos", "Repos"),
];

#[component]
pub fn PackageList(ctx: StoreSignal) -> Element {
    let mut ctx = ctx;

    let state = ctx.read();
    let rows: Vec<_> = state.filtered().cloned().collect();
    let search = state.search.clone();
    let ns_filter = state.namespace_filter;
    drop(state);

    rsx! {
        div { class: "pkg-list",

            // Namespace filter tabs
            div { class: "pkg-list__ns-tabs",
                button {
                    class: if ns_filter.is_none() { "ns-tab ns-tab--active" } else { "ns-tab" },
                    onclick: move |_| ctx.write().namespace_filter = None,
                    "All"
                }
                for (ns_id, ns_label) in NAMESPACES {
                    button {
                        class: if ns_filter == Some(ns_id) { "ns-tab ns-tab--active" } else { "ns-tab" },
                        onclick: {
                            let ns_id: &'static str = ns_id;
                            move |_| ctx.write().namespace_filter = Some(ns_id)
                        },
                        "{ns_label}"
                    }
                }
            }

            // Search bar
            div { class: "pkg-list__search",
                input {
                    r#type: "search",
                    placeholder: "Search packages…",
                    class: "search-input",
                    value: search,
                    oninput: move |e| ctx.write().search = e.value(),
                }
            }

            // Package rows
            div { class: "pkg-list__rows",
                if rows.is_empty() {
                    div { class: "pkg-list__empty", "No packages found." }
                } else {
                    for row in &rows {
                        { row.list_item(ctx) }
                    }
                }
            }
        }
    }
}
