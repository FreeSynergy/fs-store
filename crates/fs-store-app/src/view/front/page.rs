//! Pattern: Adapter (the page of one package: icon, name, version with date, the buttons, and the blocks of facts: size,
//! origin and signature level, capabilities, pictures, changes per version, description). A fact the catalog does
//! not give reads "unknown". Capabilities are kept apart from rights; nothing is loaded from the wiki or the network.

use std::sync::Arc;

use fs_components::blinc::cn::components::button::ButtonVariant;
use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;

use crate::launcher::Launcher;
use crate::model::{
    CapLine, CardState, DescriptionView, NeedsState, PackagePage, ReleaseLine, StoreModel,
};
use crate::view::card::sized_logo;
use crate::view::shell::{Shared, button, locked, rebuild};
use crate::words;
use crate::words_front as w;

use super::widgets::{badge, fact, heading};

/// Width of the column of facts.
const SIDE_W: f32 = 330.0;

fn tone(state: NeedsState) -> &'static str {
    match state {
        NeedsState::None | NeedsState::Met => "fs-store-badge-ok",
        NeedsState::Choice => "fs-store-badge-warn",
        NeedsState::Missing => "fs-store-badge-bad",
    }
}

fn caps_block(title: &str, lines: &[CapLine], answered: bool) -> Div {
    let mut block = div().flex_col().gap(1.0).child(heading(title));
    if lines.is_empty() {
        return block.child(text(w::PAGE_NOTHING).class("fs-store-sub"));
    }
    for l in lines {
        let mut row = div()
            .flex_row()
            .items_center()
            .gap(2.0)
            .child(text(l.text.clone()));
        if answered {
            row = row
                .child(badge(
                    match l.state {
                        NeedsState::Missing => w::NEEDS_MISSING,
                        NeedsState::Choice => w::NEEDS_CHOICE,
                        _ => w::NEEDS_MET,
                    },
                    tone(l.state),
                ))
                .child(text(l.answer.clone()).class("fs-store-sub"));
        }
        block = block.child(row);
    }
    block
}

fn release(r: &ReleaseLine) -> Div {
    div()
        .flex_col()
        .gap(0.5)
        .child(
            div()
                .flex_row()
                .gap(2.0)
                .child(text(r.version.clone()).bold())
                .child(text(r.date.clone()).class("fs-store-sub")),
        )
        .child(text(if r.text.is_empty() {
            "No text for this version.".to_owned()
        } else {
            r.text.clone()
        }))
}

fn actions(p: &PackagePage, busy: bool, shared: &Shared, launcher: &Arc<dyn Launcher>) -> Div {
    let name = p.card.name.clone();
    let mut row = div().flex_row().items_center().gap(2.0);
    let ask = |label: &str, id: &str, variant: ButtonVariant| {
        let (shared, name) = (Arc::clone(shared), name.clone());
        button(&format!("{id}-{name}"), label, variant, !busy).on_click(move |_| {
            locked(&shared).ask_install(&name);
            rebuild();
        })
    };
    match p.card.state {
        CardState::NotInstalled => {
            row = row.child(ask(
                words::BUTTON_INSTALL,
                "page-install",
                ButtonVariant::Primary,
            ));
        }
        CardState::UpdateAvailable => {
            row = row.child(ask(
                words::BUTTON_UPDATE,
                "page-update",
                ButtonVariant::Primary,
            ));
        }
        CardState::Installed => {}
    }
    if p.card.installed.is_some() {
        let (open, remove) = (Arc::clone(shared), Arc::clone(shared));
        let (launcher, n1, n2) = (Arc::clone(launcher), name.clone(), name);
        row = row
            .child(
                button(
                    &format!("page-open-{n1}"),
                    words::BUTTON_OPEN,
                    ButtonVariant::Secondary,
                    true,
                )
                .on_click(move |_| {
                    locked(&open).open(&n1, None, launcher.as_ref());
                    rebuild();
                }),
            )
            .child(
                button(
                    &format!("page-remove-{n2}"),
                    words::MORE_UNINSTALL,
                    ButtonVariant::Outline,
                    !busy,
                )
                .on_click(move |_| {
                    locked(&remove).ask_remove_preview(&n2);
                    rebuild();
                }),
            );
    }
    row
}

fn side(p: &PackagePage) -> Div {
    let mut origin = div().flex_col().gap(1.0).child(heading(w::PAGE_ORIGIN));
    for (label, value) in &p.origin {
        origin = origin.child(fact(label, value));
    }
    if !p.source_code.is_empty() {
        origin = origin.child(fact(w::ORIGIN_CODE, &p.source_code));
    }
    div()
        .class("fs-panel")
        .w(SIDE_W)
        .flex_shrink_0()
        .flex_col()
        .gap(4.0)
        .child(
            div()
                .flex_col()
                .gap(1.0)
                .child(heading(w::PAGE_VERSION))
                .child(text(p.version.clone()))
                .child(fact(w::PAGE_SIZE, &p.size)),
        )
        .child(origin)
        .child(
            div()
                .flex_col()
                .gap(1.0)
                .child(heading(w::PAGE_SIGNATURE))
                .child(
                    div()
                        .flex_row()
                        .child(badge(p.trust_word, "fs-store-badge-quiet")),
                )
                .child(text(p.trust_text).class("fs-store-sub")),
        )
        .child(caps_block(
            w::PAGE_PROVIDES,
            &p.capabilities.provides,
            false,
        ))
        .child(caps_block(w::PAGE_REQUIRES, &p.capabilities.requires, true))
        .child(caps_block(
            w::PAGE_RECOMMENDS,
            &p.capabilities.recommends,
            true,
        ))
}

fn main(p: &PackagePage, shared: &Shared, launcher: &Arc<dyn Launcher>) -> Div {
    let description = match &p.description {
        DescriptionView::Text(t) => text(t.clone()),
        DescriptionView::Wiki(id) => text(words::fill(w::DESCRIPTION_WIKI, &[("id", id)])),
    };
    let mut pictures = div().flex_col().gap(1.0).child(heading(w::PAGE_PICTURES));
    for (caption, license) in &p.pictures {
        let name = if caption.is_empty() {
            "Picture"
        } else {
            caption.as_str()
        };
        pictures = pictures.child(text(format!("{name} - {license}")));
    }
    pictures = pictures.child(text(p.pictures_note.clone()).class("fs-store-sub"));
    let mut changes = div().flex_col().gap(3.0).child(heading(w::PAGE_CHANGES));
    if p.releases.is_empty() {
        changes = changes.child(text(w::CHANGES_NONE).class("fs-store-sub"));
    }
    for r in &p.releases {
        changes = changes.child(release(r));
    }
    let mut links = div().flex_row().gap(2.0);
    if !p.homepage.is_empty() {
        let (shared, name, launcher) = (
            Arc::clone(shared),
            p.card.name.clone(),
            Arc::clone(launcher),
        );
        links = links.child(
            button(
                "page-website",
                w::BUTTON_WEBSITE,
                ButtonVariant::Outline,
                true,
            )
            .on_click(move |_| {
                locked(&shared).open_repository(&name, launcher.as_ref());
                rebuild();
            }),
        );
    }
    div()
        .flex_col()
        .flex_grow()
        .min_w(0.0)
        .gap(5.0)
        .child(
            div()
                .flex_col()
                .gap(1.0)
                .child(heading(w::PAGE_DESCRIPTION))
                .child(description),
        )
        .child(links)
        .child(changes)
        .child(pictures)
}

/// The page of `name`; a sentence when it is gone.
pub(super) fn page(
    model: &StoreModel,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
    name: &str,
) -> Div {
    let back = Arc::clone(shared);
    let Some(p) = model.package_page(name) else {
        return div().flex_col().child(text(words::NO_DESCRIPTION));
    };
    let mut title = div()
        .flex_col()
        .flex_grow()
        .min_w(0.0)
        .gap(1.0)
        .child(text(p.card.title.clone()).size(22.0).bold())
        .child(text(format!("{} - {}", p.kind, p.version)).class("fs-store-sub"));
    if p.icon_missing {
        title = title.child(text(w::ICON_MISSING).class("fs-store-sub"));
    }
    div()
        .flex_col()
        .gap(4.0)
        .child(div().child(
            button("page-back", w::PAGE_BACK, ButtonVariant::Ghost, true).on_click(move |_| {
                locked(&back).close_page();
                rebuild();
            }),
        ))
        .child(
            div()
                .flex_row()
                .items_center()
                .gap(4.0)
                .child(sized_logo(&p.card, 72.0, 18.0, 34.0))
                .child(title)
                .child(actions(&p, model.is_removing(), shared, launcher)),
        )
        .child(
            div()
                .flex_row()
                .gap(6.0)
                .child(main(&p, shared, launcher))
                .child(side(&p)),
        )
}
