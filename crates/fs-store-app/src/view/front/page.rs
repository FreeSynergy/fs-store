//! Pattern: Adapter (the page of one package in the Craft look: icon, name in two tones, version with date, the buttons, and the
//! blocks of facts: size, origin and signature level, capabilities, pictures, changes per version, description). A fact the catalog
//! does not give reads "unknown". Capabilities are kept apart from rights; nothing is loaded from the wiki or the network.

use std::sync::Arc;

use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_components_styles::styles::craft_sheet::caps;

use crate::launcher::Launcher;
use crate::model::{
    CapLine, CardState, DescriptionView, NeedsState, PackagePage, ReleaseLine, StoreModel,
};
use crate::view::card::sized_logo;
use crate::view::shell::{Shared, locked, rebuild};
use crate::words;
use crate::words_front as w;

use fs_components_widgets::craft::{Act, BadgeTone, Look, badge, button, fact, heading, label_dim};

/// Width of the column of facts.
const SIDE_W: f32 = 330.0;

fn tone(state: NeedsState) -> BadgeTone {
    match state {
        NeedsState::None | NeedsState::Met => BadgeTone::Ok,
        NeedsState::Choice => BadgeTone::Warn,
        NeedsState::Missing => BadgeTone::Bad,
    }
}

fn caps_block(title: &str, lines: &[CapLine], answered: bool) -> Div {
    let mut block = div().flex_col().gap(1.5).child(heading(title));
    if lines.is_empty() {
        return block.child(text(w::PAGE_NOTHING).class("craft-sub"));
    }
    for l in lines {
        let mut row = div()
            .flex_row()
            .flex_wrap()
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
                .child(text(l.answer.clone()).class("craft-sub"));
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
                .child(text(r.date.clone()).class("craft-sub")),
        )
        .child(text(if r.text.is_empty() {
            "No text for this version.".to_owned()
        } else {
            r.text.clone()
        }))
}

fn act(shared: &Shared, f: impl Fn(&mut StoreModel) + Send + Sync + 'static) -> Act {
    let shared = Arc::clone(shared);
    Arc::new(move || {
        f(&mut locked(&shared));
        rebuild();
    })
}

fn actions(p: &PackagePage, busy: bool, shared: &Shared, launcher: &Arc<dyn Launcher>) -> Div {
    let name = p.card.name.clone();
    let mut row = div().flex_row().items_center().gap(2.0);
    let ask = |label: &str, id: &str| {
        let name = name.clone();
        button(
            &format!("{id}-{name}"),
            label,
            Look::Fill,
            !busy && p.card.install_blocked.is_none(),
            act(shared, move |m| m.ask_install(&name)),
        )
    };
    match p.card.state {
        CardState::NotInstalled => row = row.child(ask(words::BUTTON_INSTALL, "page-install")),
        CardState::UpdateAvailable => row = row.child(ask(words::BUTTON_UPDATE, "page-update")),
        CardState::Installed => {}
    }
    if p.card.installed.is_some() {
        let (launcher, n1, n2) = (Arc::clone(launcher), name.clone(), name);
        let open = Arc::clone(shared);
        row = row
            .child(button(
                &format!("page-open-{n1}"),
                words::BUTTON_OPEN,
                Look::Line,
                p.card.open_blocked.is_none(),
                Arc::new(move || {
                    locked(&open).open(&n1, None, launcher.as_ref());
                    rebuild();
                }),
            ))
            .child(button(
                &format!("page-remove-{n2}"),
                words::MORE_UNINSTALL,
                Look::Line,
                !busy,
                act(shared, move |m| m.ask_remove_preview(&n2)),
            ));
    }
    row
}

fn side(p: &PackagePage) -> Div {
    let mut origin = div().flex_col().gap(1.5).child(heading(w::PAGE_ORIGIN));
    for (name, value) in &p.origin {
        origin = origin.child(fact(name, value));
    }
    if !p.source_code.is_empty() {
        origin = origin.child(fact(w::ORIGIN_CODE, &p.source_code));
    }
    div()
        .class("craft-panel")
        .w(SIDE_W)
        .flex_shrink_0()
        .flex_col()
        .gap(5.0)
        .p(5.0)
        .child(
            div()
                .flex_col()
                .gap(1.5)
                .child(heading(w::PAGE_VERSION))
                .child(text(p.version.clone()))
                .child(fact(w::PAGE_SIZE, &p.size)),
        )
        .child(origin)
        .child(
            div()
                .flex_col()
                .gap(1.5)
                .child(heading(w::PAGE_SIGNATURE))
                .child(
                    div()
                        .flex_row()
                        .child(badge(p.trust_word, BadgeTone::Quiet)),
                )
                .child(text(p.trust_text).class("craft-sub")),
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

fn main(p: &PackagePage) -> Div {
    let description = match &p.description {
        DescriptionView::Text(t) => text(t.clone()),
        DescriptionView::Wiki(id) => text(words::fill(w::DESCRIPTION_WIKI, &[("id", id)])),
    };
    let mut pictures = div().flex_col().gap(1.5).child(heading(w::PAGE_PICTURES));
    for (caption, license) in &p.pictures {
        let name = if caption.is_empty() {
            "Picture"
        } else {
            caption.as_str()
        };
        pictures = pictures.child(text(format!("{name} - {license}")));
    }
    pictures = pictures.child(text(p.pictures_note.clone()).class("craft-sub"));
    let mut changes = div().flex_col().gap(3.0).child(heading(w::PAGE_CHANGES));
    if p.releases.is_empty() {
        changes = changes.child(text(w::CHANGES_NONE).class("craft-sub"));
    }
    for r in &p.releases {
        changes = changes.child(release(r));
    }
    div()
        .flex_col()
        .flex_grow()
        .min_w(0.0)
        .gap(7.0)
        .child(
            div()
                .flex_col()
                .gap(1.5)
                .child(heading(w::PAGE_DESCRIPTION))
                .child(description),
        )
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
    let Some(p) = model.package_page(name) else {
        return div().flex_col().child(text(words::NO_DESCRIPTION));
    };
    let mut title = div()
        .flex_col()
        .flex_grow()
        .min_w(0.0)
        .gap(1.0)
        .child(text(p.card.title.clone()).size(24.0).bold())
        .child(text(caps(&format!("{} - {}", p.kind, p.version))).class("craft-caps-dim"));
    if p.icon_missing {
        title = title.child(label_dim(w::ICON_MISSING));
    }
    // A locked Open says why, in words, right under the name
    if let Some(why) = p.card.open_blocked.filter(|_| p.card.installed.is_some()) {
        title = title.child(text(why).class("craft-sub"));
    }
    if let Some(why) = p.card.install_blocked {
        title = title.child(text(why).class("craft-sub"));
    }
    let mut links = div().flex_row().gap(2.0);
    if !p.homepage.is_empty() {
        let n = p.card.name.clone();
        let l = Arc::clone(launcher);
        let s = Arc::clone(shared);
        links = links.child(button(
            "page-website",
            w::BUTTON_WEBSITE,
            Look::Line,
            true,
            Arc::new(move || {
                locked(&s).open_repository(&n, l.as_ref());
                rebuild();
            }),
        ));
    }
    div()
        .flex_col()
        .gap(5.0)
        .child(div().flex_row().child(button(
            "page-back",
            w::PAGE_BACK,
            Look::Line,
            true,
            act(shared, StoreModel::close_page),
        )))
        .child(
            div()
                .flex_row()
                .items_center()
                .gap(5.0)
                .child(sized_logo(&p.card, 72.0, 14.0, 34.0))
                .child(title)
                .child(actions(
                    &p,
                    model.is_removing() || p.card.working.is_some(),
                    shared,
                    launcher,
                )),
        )
        .child(
            div()
                .flex_row()
                .gap(7.0)
                .child(
                    div()
                        .flex_col()
                        .flex_grow()
                        .min_w(0.0)
                        .gap(5.0)
                        .child(links)
                        .child(main(&p)),
                )
                .child(side(&p)),
        )
}
