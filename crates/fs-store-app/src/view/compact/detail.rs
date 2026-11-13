//! Pattern: Adapter (the detail pane of variant C: the head with the logo, the name and the buttons; the progress of a running job;
//! the question before any work; then the facts of the catalog in blocks). A fact the catalog does not give reads "unknown".

use std::sync::Arc;

use fs_components::blinc::cn::components::button::ButtonVariant;
use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;

use crate::launcher::Launcher;
use crate::model::{CapLine, CardState, DescriptionView, NeedsState, PackagePage, StoreModel};
use crate::view::card::logo_plain;
use crate::view::shell::{Shared, button, locked, rebuild};
use crate::words;
use crate::words_front as w;

use super::preview::preview;

fn block(title: &str, body: Div) -> Div {
    div()
        .class("fs-c-card")
        .flex_col()
        .gap(2.0)
        .child(text(title.to_owned()).class("fs-c-h"))
        .child(body)
}

fn fact(name: &str, value: &str) -> Div {
    div()
        .flex_row()
        .gap(3.0)
        .child(
            div()
                .w(140.0)
                .flex_shrink_0()
                .child(text(name.to_owned()).class("fs-c-sub")),
        )
        .child(div().flex_grow().min_w(0.0).child(text(value.to_owned())))
}

fn tone(state: NeedsState) -> &'static str {
    match state {
        NeedsState::None | NeedsState::Met => "fs-c-ok",
        NeedsState::Choice => "fs-c-warn",
        NeedsState::Missing => "fs-c-bad",
    }
}

fn caps(lines: &[CapLine], answered: bool) -> Div {
    let mut col = div().flex_col().gap(1.0);
    if lines.is_empty() {
        return col.child(text(w::PAGE_NOTHING).class("fs-c-sub"));
    }
    for l in lines {
        let mut row = div()
            .flex_row()
            .flex_wrap()
            .gap(2.0)
            .child(text(l.text.clone()));
        if answered {
            row = row.child(text(l.answer.clone()).class(tone(l.state)));
        }
        col = col.child(row);
    }
    col
}

fn buttons(p: &PackagePage, busy: bool, shared: &Shared, launcher: &Arc<dyn Launcher>) -> Div {
    let name = p.card.name.clone();
    let mut row = div().flex_row().gap(2.0);
    let ask = |label: &str, id: &str, variant: ButtonVariant| {
        let (shared, name) = (Arc::clone(shared), name.clone());
        button(
            &format!("{id}-{name}"),
            label,
            variant,
            !busy && p.card.install_blocked.is_none(),
        )
        .on_click(move |_| {
            locked(&shared).ask_install(&name);
            rebuild();
        })
    };
    match p.card.state {
        CardState::NotInstalled => {
            row = row.child(ask(
                words::BUTTON_INSTALL,
                "c-install",
                ButtonVariant::Primary,
            ));
        }
        CardState::UpdateAvailable => {
            row = row.child(ask(
                words::BUTTON_UPDATE,
                "c-update",
                ButtonVariant::Primary,
            ));
        }
        CardState::Installed => {}
    }
    if p.card.installed.is_some() {
        let (open, remove, launcher) =
            (Arc::clone(shared), Arc::clone(shared), Arc::clone(launcher));
        let (n1, n2) = (name.clone(), name);
        row = row
            .child(
                button(
                    &format!("c-open-{n1}"),
                    words::BUTTON_OPEN,
                    ButtonVariant::Secondary,
                    p.card.open_blocked.is_none(),
                )
                .on_click(move |_| {
                    locked(&open).open(&n1, None, launcher.as_ref());
                    rebuild();
                }),
            )
            .child(
                button(
                    &format!("c-remove-{n2}"),
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

/// While a job runs: the sentence, the bar, and Cancel when it can stop with nothing half done.
fn running(p: &PackagePage, shared: &Shared, width: f32) -> Option<Div> {
    let (share, line) = p.card.progress()?;
    let (shared, name) = (Arc::clone(shared), p.card.name.clone());
    let bar_w = (width - 40.0 - 100.0).max(60.0);
    Some(
        div()
            .class("fs-c-card")
            .flex_col()
            .gap(2.0)
            .child(text(line))
            .child(
                div()
                    .flex_row()
                    .items_center()
                    .gap(3.0)
                    .child(
                        div().class("fs-c-track").w(bar_w).child(
                            div()
                                .h(6.0)
                                .w((bar_w * share).max(2.0))
                                .bg(fs_components::blinc::core::Color::rgb(0.11, 0.6, 0.95))
                                .rounded(3.0),
                        ),
                    )
                    .child(
                        button(
                            &format!("c-cancel-{name}"),
                            w::CANCEL_TIP,
                            ButtonVariant::Outline,
                            p.card.can_cancel,
                        )
                        .on_click(move |_| {
                            locked(&shared).cancel_job(&name);
                            rebuild();
                        }),
                    ),
            ),
    )
}

/// The facts of the version block: size, origin, signature.
fn facts_of(p: &PackagePage) -> Div {
    let mut facts = div().flex_col().gap(1.0).child(fact(w::PAGE_SIZE, &p.size));
    for (name, value) in &p.origin {
        facts = facts.child(fact(name, value));
    }
    if !p.source_code.is_empty() {
        facts = facts.child(fact(w::ORIGIN_CODE, &p.source_code));
    }
    facts = facts
        .child(fact(w::PAGE_SIGNATURE, p.trust_word))
        .child(text(p.trust_text).class("fs-c-sub"));
    facts
}

/// The changes per version.
fn changes_of(p: &PackagePage) -> Div {
    let mut changes = div().flex_col().gap(3.0);
    if p.releases.is_empty() {
        changes = changes.child(text(w::CHANGES_NONE).class("fs-c-sub"));
    }
    for r in &p.releases {
        changes = changes.child(
            div()
                .flex_col()
                .child(
                    div()
                        .flex_row()
                        .gap(2.0)
                        .child(text(r.version.clone()).bold())
                        .child(text(r.date.clone()).class("fs-c-sub")),
                )
                .child(text(if r.text.is_empty() {
                    "No text for this version.".to_owned()
                } else {
                    r.text.clone()
                })),
        );
    }
    changes
}

/// The detail pane, `width` x `height`.
pub(super) fn pane(
    model: &StoreModel,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
    (width, height): (f32, f32),
) -> Div {
    let mut pane = div()
        .class("fs-c-detail")
        .flex_col()
        .gap(4.0)
        .w(width)
        .h(height)
        .min_w(0.0);
    pane = pane.child(crate::view::notices::strip(model, shared));
    let Some(p) = model.package_page_selected() else {
        return pane.child(text(w::NOTHING_MATCHES).class("fs-c-sub"));
    };
    let head = div()
        .class("fs-c-head")
        .flex_row()
        .items_center()
        .gap(4.0)
        .child(logo_plain(&p.card, 64.0))
        .child(
            div()
                .flex_col()
                .flex_grow()
                .min_w(0.0)
                .gap(1.0)
                .child(text(p.card.title.clone()).size(22.0).bold())
                .child(text(format!("{} \u{b7} {}", p.kind, p.version)).class("fs-c-sub")),
        )
        .child(buttons(
            &p,
            model.is_removing() || p.card.working.is_some(),
            shared,
            launcher,
        ));
    pane = pane.child(head);
    if let Some(why) = p.card.open_blocked.filter(|_| p.card.installed.is_some()) {
        pane = pane.child(text(why).class("fs-c-sub"));
    }
    if let Some(why) = &p.card.install_blocked {
        pane = pane.child(text(why).class("fs-c-sub"));
    }
    if let Some(r) = running(&p, shared, width) {
        pane = pane.child(r);
    }
    if let Some(q) = preview(model, shared) {
        pane = pane.child(q);
    }
    let description = match &p.description {
        DescriptionView::Text(t) => text(t.clone()),
        DescriptionView::Wiki(id) => text(words::fill(w::DESCRIPTION_WIKI, &[("id", id)])),
    };
    pane = pane
        .child(block(w::PAGE_DESCRIPTION, div().child(description)))
        .child(block(w::PAGE_VERSION, facts_of(&p)));
    // Only the blocks that have something get a box; the empty ones are one line (below), not three boxes that say "nothing"
    let caps_blocks = [
        (w::PAGE_PROVIDES, &p.capabilities.provides, false),
        (w::PAGE_REQUIRES, &p.capabilities.requires, true),
        (w::PAGE_RECOMMENDS, &p.capabilities.recommends, true),
    ];
    for (title, lines, answered) in caps_blocks {
        if !lines.is_empty() {
            pane = pane.child(block(title, caps(lines, answered)));
        }
    }
    pane = pane.child(block(w::PAGE_CHANGES, changes_of(&p)));
    if !p.pictures.is_empty() {
        pane = pane.child(block(
            w::PAGE_PICTURES,
            div().child(text(p.pictures_note.clone()).class("fs-c-sub")),
        ));
    }
    if let Some(line) = &p.nothing_line {
        pane = pane.child(text(line.clone()).class("fs-c-sub"));
    }
    pane
}
