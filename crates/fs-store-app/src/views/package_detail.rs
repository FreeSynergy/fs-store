// views/package_detail.rs — detail view for a selected package.

use dioxus::prelude::*;

use crate::context::StoreSignal;
use crate::view::PackageView;

#[component]
pub fn PackageDetail(ctx: StoreSignal) -> Element {
    let state = ctx.read();
    let Some(pkg) = state.selected().cloned() else {
        return rsx! {
            div { class: "pkg-detail pkg-detail--empty",
                p { "Select a package to see details." }
            }
        };
    };
    drop(state);

    pkg.detail_panel(ctx)
}
