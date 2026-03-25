// view.rs — PackageView trait: each domain object renders itself.
//
// OOP rule: View-Trait statt ViewRenderer.
// Components call pkg.list_item() / pkg.detail_panel() — zero logic in RSX.

use dioxus::prelude::*;

use crate::context::{PackageRow, StoreSignal};

// ── PackageView trait ─────────────────────────────────────────────────────────

pub trait PackageView {
    /// Render a compact one-line row for the package list.
    fn list_item(&self, ctx: StoreSignal) -> Element;

    /// Render a full detail panel for the selected package.
    fn detail_panel(&self, ctx: StoreSignal) -> Element;
}

// ── impl PackageView for PackageRow ───────────────────────────────────────────

impl PackageView for PackageRow {
    fn list_item(&self, mut ctx: StoreSignal) -> Element {
        let id = self.id.clone();
        let name = self.name.clone();
        let version = self.version.clone();
        let summary = self.summary.clone();
        let ns = self.namespace;
        let is_installed = self.is_installed;
        let is_selected = ctx.read().selected_id.as_deref() == Some(self.id.as_str());

        rsx! {
            div {
                class: if is_selected { "pkg-row pkg-row--selected" } else { "pkg-row" },
                onclick: move |_| ctx.write().selected_id = Some(id.clone()),

                div { class: "pkg-row__left",
                    div { class: "pkg-row__name", "{name}" }
                    div { class: "pkg-row__summary", "{summary}" }
                }
                div { class: "pkg-row__right",
                    span { class: "pkg-row__ns", "{ns}" }
                    span { class: "pkg-row__version", "v{version}" }
                    if is_installed {
                        span { class: "pkg-row__badge", "✓" }
                    }
                }
            }
        }
    }

    fn detail_panel(&self, mut ctx: StoreSignal) -> Element {
        let name = self.name.clone();
        let id = self.id.clone();
        let version = self.version.clone();
        let ns = self.namespace;
        let summary = self.summary.clone();
        let description = self.description.clone();
        let tags = self.tags.clone();
        let is_installed = self.is_installed;
        let installed_version = self.installed_version.clone();
        let has_update = self.has_update;

        rsx! {
            div { class: "pkg-detail",

                // Header
                div { class: "pkg-detail__header",
                    button {
                        class: "pkg-detail__back",
                        onclick: move |_| ctx.write().selected_id = None,
                        "← Back"
                    }
                    div { class: "pkg-detail__title",
                        h2 { "{name}" }
                        span { class: "pkg-detail__id", "{id}" }
                    }
                }

                // Meta row
                div { class: "pkg-detail__meta",
                    span { class: "pkg-detail__ns", "{ns}" }
                    span { class: "pkg-detail__version", "v{version}" }
                    if is_installed {
                        span { class: "pkg-detail__installed", "Installed" }
                    }
                    if has_update {
                        span { class: "pkg-detail__update", "Update available" }
                    }
                }

                // Summary + description
                p { class: "pkg-detail__summary", "{summary}" }
                if !description.is_empty() {
                    div { class: "pkg-detail__description",
                        for line in description.lines() {
                            p { "{line}" }
                        }
                    }
                }

                // Tags
                if !tags.is_empty() {
                    div { class: "pkg-detail__tags",
                        for tag in &tags {
                            span { class: "pkg-detail__tag", "{tag}" }
                        }
                    }
                }

                // Install status + action
                div { class: "pkg-detail__actions",
                    if is_installed {
                        div { class: "pkg-detail__installed-info",
                            if let Some(v) = &installed_version {
                                span { "Installed: v{v}" }
                            }
                            button {
                                class: "btn btn--danger",
                                disabled: true,
                                "Remove (coming soon)"
                            }
                        }
                    } else {
                        button {
                            class: "btn btn--primary",
                            disabled: true,
                            "Install (coming soon)"
                        }
                    }
                }
            }
        }
    }
}
