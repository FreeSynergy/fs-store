// views/installed.rs — list of locally installed packages.

use dioxus::prelude::*;

use crate::context::StoreSignal;
use crate::view::PackageView;

#[component]
pub fn InstalledList(ctx: StoreSignal) -> Element {
    let mut ctx = ctx;
    let state = ctx.read();
    let installed: Vec<_> = state.installed().cloned().collect();
    drop(state);

    rsx! {
        div { class: "installed-list",
            h2 { class: "installed-list__title", "Installed Packages" }

            if installed.is_empty() {
                div { class: "installed-list__empty",
                    p { "No packages installed yet." }
                    p { class: "installed-list__hint",
                        "Browse the "
                        button {
                            class: "link-btn",
                            onclick: move |_| {
                                ctx.write().selected_id = None;
                            },
                            "Store"
                        }
                        " to find packages."
                    }
                }
            } else {
                for row in &installed {
                    { row.list_item(ctx) }
                }
                p { class: "installed-list__count",
                    "{installed.len()} package(s) installed"
                }
            }
        }
    }
}
