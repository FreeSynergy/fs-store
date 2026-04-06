// view.rs — PackageView trait: each domain object renders itself.
//
// OOP rule: View-Trait statt ViewRenderer.
// Components call pkg.list_item() / pkg.detail_panel() — zero logic in RSX.

use dioxus::prelude::*;
use fs_store::{ApiEndpoint, StoragePaths};

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
        let is_incomplete = self.is_incomplete;
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
                    if is_incomplete {
                        span { class: "pkg-row__badge pkg-row__badge--incomplete", "!" }
                    }
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
        let is_incomplete = self.is_incomplete;
        let license = self.license.clone();
        let homepage = self.homepage.clone();
        let storage = self.storage.clone();
        let api_endpoints = self.api_endpoints.clone();
        let screenshots = self.screenshots.clone();
        let store_available = ctx.read().store_available;

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
                    if !license.is_empty() {
                        span { class: "pkg-detail__license", "{license}" }
                    }
                    if is_installed {
                        span { class: "pkg-detail__installed", "Installed" }
                    }
                    if has_update {
                        span { class: "pkg-detail__update", "Update available" }
                    }
                    if is_incomplete {
                        span { class: "pkg-detail__incomplete", "Incomplete metadata" }
                    }
                }
                if let Some(ref url) = homepage {
                    p { class: "pkg-detail__homepage",
                        a { href: "{url}", target: "_blank", "{url}" }
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

                // Screenshots (only when declared)
                if !screenshots.is_empty() {
                    {screenshot_strip(&screenshots)}
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
                                disabled: !store_available,
                                onclick: move |_| {
                                    // Remove wizard is wired in G2 (Dioxus → iced migration).
                                    let _ = &id;
                                },
                                "Remove"
                            }
                        }
                    } else {
                        button {
                            class: "btn btn--primary",
                            disabled: !store_available,
                            onclick: move |_| {
                                // Install wizard is wired in G2 (Dioxus → iced migration).
                                let _ = &id;
                            },
                            "Install"
                        }
                    }
                }

                // Storage tab (only when paths are declared)
                if !storage.is_empty() {
                    {storage_tab(&storage)}
                }

                // API tab (only when endpoints are declared)
                if !api_endpoints.is_empty() {
                    {api_tab(&api_endpoints)}
                }
            }
        }
    }
}

// ── Sub-views ─────────────────────────────────────────────────────────────────

fn screenshot_strip(screenshots: &[String]) -> Element {
    rsx! {
        div { class: "pkg-detail__screenshots",
            h3 { class: "pkg-detail__tab-title", "Screenshots" }
            div { class: "pkg-detail__screenshot-row",
                for path in screenshots {
                    img {
                        class: "pkg-detail__screenshot",
                        src: "{path}",
                        alt: "Screenshot",
                        loading: "lazy",
                    }
                }
            }
        }
    }
}

fn storage_tab(storage: &StoragePaths) -> Element {
    rsx! {
        div { class: "pkg-detail__tab pkg-detail__tab--storage",
            h3 { class: "pkg-detail__tab-title", "Storage" }
            table { class: "pkg-detail__storage-table",
                if let Some(ref path) = storage.user {
                    tr {
                        th { "User" }
                        td { class: "pkg-detail__path", "{path}" }
                    }
                }
                if let Some(ref path) = storage.global {
                    tr {
                        th { "Global" }
                        td { class: "pkg-detail__path", "{path}" }
                    }
                }
                if let Some(ref path) = storage.config {
                    tr {
                        th { "Config" }
                        td { class: "pkg-detail__path", "{path}" }
                    }
                }
                if let Some(ref path) = storage.cache {
                    tr {
                        th { "Cache" }
                        td { class: "pkg-detail__path", "{path}" }
                    }
                }
            }
        }
    }
}

fn api_tab(endpoints: &[ApiEndpoint]) -> Element {
    rsx! {
        div { class: "pkg-detail__tab pkg-detail__tab--api",
            h3 { class: "pkg-detail__tab-title", "API" }
            for ep in endpoints {
                div { class: "pkg-detail__api-endpoint",
                    div { class: "pkg-detail__api-base",
                        span { class: "pkg-detail__api-proto", "{ep.proto}://" }
                        span { class: "pkg-detail__api-path", "{ep.base}" }
                        if let Some(port) = ep.port {
                            span { class: "pkg-detail__api-port", ":{port}" }
                        }
                    }
                    if !ep.description.is_empty() {
                        p { class: "pkg-detail__api-desc", "{ep.description}" }
                    }
                }
            }
        }
    }
}
