// app.rs — Root App component.
//
// Layout:
//   ┌────────────────────────────────────────────────────┐
//   │  FreeSynergy Store           by KalEl              │
//   ├──────────────┬─────────────────────────────────────┤
//   │  [Browse]    │                                     │
//   │  [Installed] │   PackageList / PackageDetail       │
//   │              │                                     │
//   └──────────────┴─────────────────────────────────────┘
//
// State: StoreContext (Signal<StoreState>) provided here, read everywhere.

use dioxus::prelude::*;

use crate::context::{use_store_context, StoreSignal};
use crate::views::installed::InstalledList;
use crate::views::package_detail::PackageDetail;
use crate::views::package_list::PackageList;
use crate::views::updates::UpdatesList;

// ── Sidebar tab ───────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum Tab {
    Browse,
    Installed,
    Updates,
}

// ── App ───────────────────────────────────────────────────────────────────────

pub fn App() -> Element {
    let ctx: StoreSignal = use_store_context();
    let mut tab = use_signal(|| Tab::Browse);

    let state = ctx.read();
    let loading = state.loading;
    let error = state.error.clone();
    let pkg_count = state.rows.len();
    let has_selection = state.selected_id.is_some();
    let update_count = state.with_updates().count();
    drop(state);

    rsx! {
        style { {CSS} }

        div { class: "fs-app",

            // ── Header ───────────────────────────────────────────────────────
            header { class: "fs-header",
                span { class: "fs-header__title", "FreeSynergy Store" }
                span { class: "fs-header__by", "by KalEl" }
                if !loading && pkg_count > 0 {
                    span { class: "fs-header__count", "{pkg_count} packages" }
                }
            }

            // ── Body ─────────────────────────────────────────────────────────
            div { class: "fs-body",

                // Sidebar
                nav { class: "fs-sidebar",
                    button {
                        class: if *tab.read() == Tab::Browse { "fs-sidebar__item fs-sidebar__item--active" } else { "fs-sidebar__item" },
                        onclick: move |_| tab.set(Tab::Browse),
                        "Browse"
                    }
                    button {
                        class: if *tab.read() == Tab::Installed { "fs-sidebar__item fs-sidebar__item--active" } else { "fs-sidebar__item" },
                        onclick: move |_| tab.set(Tab::Installed),
                        "Installed"
                    }
                    button {
                        class: if *tab.read() == Tab::Updates { "fs-sidebar__item fs-sidebar__item--active" } else { "fs-sidebar__item" },
                        onclick: move |_| tab.set(Tab::Updates),
                        if update_count > 0 {
                            "Updates ({update_count})"
                        } else {
                            "Updates"
                        }
                    }
                }

                // Content
                div { class: "fs-content",
                    if loading {
                        div { class: "fs-loading",
                            p { "Loading catalog…" }
                        }
                    } else if let Some(err) = &error {
                        div { class: "fs-error",
                            p { class: "fs-error__title", "Failed to load Store" }
                            p { class: "fs-error__msg", "{err}" }
                        }
                    } else if has_selection {
                        PackageDetail { ctx }
                    } else {
                        match *tab.read() {
                            Tab::Browse    => rsx! { PackageList { ctx } },
                            Tab::Installed => rsx! { InstalledList { ctx } },
                            Tab::Updates   => rsx! { UpdatesList { ctx } },
                        }
                    }
                }
            }
        }
    }
}

// ── CSS ───────────────────────────────────────────────────────────────────────

const CSS: &str = r"
:root {
    --bg-base:      #0f1117;
    --bg-surface:   #1a1d27;
    --bg-elevated:  #242838;
    --text-primary: #e8eaf0;
    --text-muted:   #7a7f99;
    --accent:       #00c8c8;
    --accent-hover: #00e0e0;
    --border:       #2d3148;
    --radius-sm:    4px;
    --radius-md:    8px;
    --danger:       #e05c5c;
}

* { box-sizing: border-box; margin: 0; padding: 0; }

body {
    background: var(--bg-base);
    color: var(--text-primary);
    font-family: system-ui, -apple-system, sans-serif;
    font-size: 14px;
}

/* App shell */
.fs-app { display: flex; flex-direction: column; height: 100vh; }

.fs-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 20px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
}
.fs-header__title { font-size: 16px; font-weight: 600; color: var(--accent); }
.fs-header__by    { font-size: 12px; color: var(--text-muted); }
.fs-header__count { margin-left: auto; font-size: 12px; color: var(--text-muted); }

.fs-body    { display: flex; flex: 1; overflow: hidden; }

.fs-sidebar {
    width: 140px;
    background: var(--bg-surface);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 8px;
    gap: 2px;
    flex-shrink: 0;
}
.fs-sidebar__item {
    background: none;
    border: none;
    color: var(--text-primary);
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-align: left;
    font-size: 14px;
}
.fs-sidebar__item:hover              { background: var(--bg-elevated); }
.fs-sidebar__item--active            { background: var(--bg-elevated); color: var(--accent); }

.fs-content { flex: 1; overflow: hidden; display: flex; }

/* Loading / Error */
.fs-loading, .fs-error {
    margin: auto;
    text-align: center;
    color: var(--text-muted);
    padding: 48px;
}
.fs-error__title { font-size: 18px; margin-bottom: 8px; color: var(--danger); }
.fs-error__msg   { font-size: 12px; color: var(--text-muted); max-width: 400px; }

/* Package list */
.pkg-list { display: flex; flex-direction: column; width: 100%; overflow: hidden; }

.pkg-list__ns-tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 8px 12px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
}
.ns-tab {
    background: none;
    border: 1px solid var(--border);
    color: var(--text-muted);
    padding: 4px 10px;
    border-radius: 20px;
    cursor: pointer;
    font-size: 12px;
}
.ns-tab:hover       { border-color: var(--accent); color: var(--text-primary); }
.ns-tab--active     { background: var(--accent); border-color: var(--accent); color: #000; font-weight: 600; }

.pkg-list__tag-tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 6px 12px;
    background: var(--bg-base);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    max-height: 80px;
    overflow-y: auto;
}
.tag-tab {
    background: none;
    border: 1px solid var(--border);
    color: var(--text-muted);
    padding: 3px 8px;
    border-radius: 20px;
    cursor: pointer;
    font-size: 11px;
}
.tag-tab:hover       { border-color: var(--accent); color: var(--text-primary); }
.tag-tab--active     { background: #00404040; border-color: var(--accent); color: var(--accent); }

/* Updates list */
.updates-list        { padding: 24px; overflow-y: auto; width: 100%; }
.updates-list__title { font-size: 18px; font-weight: 600; margin-bottom: 16px; }
.updates-list__empty { color: var(--text-muted); }
.updates-list__count { margin-top: 12px; font-size: 12px; color: var(--text-muted); }

.pkg-list__search { padding: 8px 12px; border-bottom: 1px solid var(--border); flex-shrink: 0; }
.search-input {
    width: 100%;
    padding: 7px 12px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: 13px;
}
.search-input:focus { outline: none; border-color: var(--accent); }

.pkg-list__rows  { flex: 1; overflow-y: auto; padding: 4px 0; }
.pkg-list__empty { padding: 32px; text-align: center; color: var(--text-muted); }

/* Package row */
.pkg-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    cursor: pointer;
    border-bottom: 1px solid var(--border);
}
.pkg-row:hover         { background: var(--bg-surface); }
.pkg-row--selected     { background: var(--bg-elevated); border-left: 3px solid var(--accent); }
.pkg-row__left         { flex: 1; min-width: 0; }
.pkg-row__name         { font-weight: 500; margin-bottom: 2px; }
.pkg-row__summary      { font-size: 12px; color: var(--text-muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.pkg-row__right        { display: flex; align-items: center; gap: 8px; flex-shrink: 0; margin-left: 12px; }
.pkg-row__ns           { font-size: 11px; color: var(--text-muted); background: var(--bg-elevated); padding: 2px 6px; border-radius: 10px; }
.pkg-row__version      { font-size: 11px; color: var(--text-muted); font-family: monospace; }
.pkg-row__badge                    { font-size: 11px; color: var(--accent); }
.pkg-row__badge--incomplete        { color: #e0a040; font-weight: 700; }

/* Package detail */
.pkg-detail         { padding: 24px; overflow-y: auto; width: 100%; }
.pkg-detail--empty  { display: flex; align-items: center; justify-content: center; width: 100%; color: var(--text-muted); }

.pkg-detail__header { display: flex; align-items: flex-start; gap: 16px; margin-bottom: 16px; }
.pkg-detail__back {
    background: none;
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 13px;
    flex-shrink: 0;
}
.pkg-detail__back:hover   { border-color: var(--accent); color: var(--accent); }
.pkg-detail__title h2     { font-size: 22px; margin-bottom: 4px; }
.pkg-detail__id           { font-size: 12px; color: var(--text-muted); font-family: monospace; }

.pkg-detail__meta         { display: flex; gap: 8px; margin-bottom: 12px; flex-wrap: wrap; }
.pkg-detail__ns           { font-size: 11px; background: var(--bg-elevated); padding: 3px 8px; border-radius: 10px; color: var(--text-muted); }
.pkg-detail__version      { font-size: 11px; background: var(--bg-elevated); padding: 3px 8px; border-radius: 10px; font-family: monospace; }
.pkg-detail__installed    { font-size: 11px; background: #1a3a1a; color: #6aba6a; padding: 3px 8px; border-radius: 10px; }
.pkg-detail__update       { font-size: 11px; background: #3a3a1a; color: #baba6a; padding: 3px 8px; border-radius: 10px; }
.pkg-detail__incomplete   { font-size: 11px; background: #3a2a0a; color: #e0a040; padding: 3px 8px; border-radius: 10px; }
.pkg-detail__license      { font-size: 11px; background: var(--bg-elevated); padding: 3px 8px; border-radius: 10px; color: var(--text-muted); }
.pkg-detail__homepage     { font-size: 12px; margin-bottom: 12px; }
.pkg-detail__homepage a   { color: var(--accent); }

.pkg-detail__summary      { font-size: 15px; margin-bottom: 12px; color: var(--text-primary); }
.pkg-detail__description  { color: var(--text-muted); margin-bottom: 16px; line-height: 1.6; }
.pkg-detail__description p { margin-bottom: 6px; }

.pkg-detail__tags         { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 20px; }
.pkg-detail__tag          { font-size: 11px; background: var(--bg-elevated); border: 1px solid var(--border); padding: 3px 8px; border-radius: 10px; color: var(--text-muted); }

.pkg-detail__actions      { margin-top: 16px; }
.pkg-detail__installed-info { display: flex; align-items: center; gap: 16px; }

/* Installed list */
.installed-list        { padding: 24px; overflow-y: auto; width: 100%; }
.installed-list__title { font-size: 18px; font-weight: 600; margin-bottom: 16px; }
.installed-list__empty { color: var(--text-muted); }
.installed-list__hint  { margin-top: 8px; font-size: 13px; }
.installed-list__count { margin-top: 12px; font-size: 12px; color: var(--text-muted); }

/* Screenshots */
.pkg-detail__screenshots      { margin-bottom: 20px; }
.pkg-detail__screenshot-row   { display: flex; gap: 10px; overflow-x: auto; padding-bottom: 4px; }
.pkg-detail__screenshot       { height: 180px; border-radius: var(--radius-md); border: 1px solid var(--border); object-fit: cover; flex-shrink: 0; }

/* Buttons */
.btn {
    padding: 8px 16px;
    border-radius: var(--radius-sm);
    border: none;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
}
.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn--primary { background: var(--accent); color: #000; }
.btn--primary:hover:not(:disabled) { background: var(--accent-hover); }
.btn--danger  { background: var(--danger); color: #fff; }

.link-btn {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font-size: inherit;
    padding: 0;
    text-decoration: underline;
}
";
