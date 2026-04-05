// views/package_list.rs — browsable list of all Store packages.
//
// Phase 5B.2: added tag-filter row so users can navigate by category tag.

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
    let tag_filter = state.tag_filter.clone();
    let all_tags = state.all_tags();
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

            // Tag filter (category navigation)
            if !all_tags.is_empty() {
                div { class: "pkg-list__tag-tabs",
                    button {
                        class: if tag_filter.is_none() { "tag-tab tag-tab--active" } else { "tag-tab" },
                        onclick: move |_| ctx.write().tag_filter = None,
                        "All tags"
                    }
                    for tag in &all_tags {
                        button {
                            class: if tag_filter.as_deref() == Some(tag) { "tag-tab tag-tab--active" } else { "tag-tab" },
                            onclick: {
                                let tag = tag.clone();
                                move |_| ctx.write().tag_filter = Some(tag.clone())
                            },
                            "{tag}"
                        }
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
