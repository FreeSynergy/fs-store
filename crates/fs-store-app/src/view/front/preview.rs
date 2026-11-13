//! Pattern: Adapter (the question before any work in the Craft look: the fixed steps in plain words, each with how far it reaches
//! and who does it, the highest risk, and the two answers). The steps are those the store core really runs.

use std::sync::Arc;

use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;

use crate::model::{Preview, PreviewAction, Risk, StoreModel};
use crate::view::job::{start_all_updates, start_job, start_uninstall};
use crate::view::shell::{Shared, locked, rebuild};
use crate::words_front as w;

use fs_components_widgets::craft::{BadgeTone, Look, badge, button, label_dim};

fn risk_tone(risk: Risk) -> BadgeTone {
    match risk {
        Risk::Look | Risk::Open => BadgeTone::Quiet,
        Risk::Start => BadgeTone::Info,
        Risk::Install => BadgeTone::Warn,
        Risk::System => BadgeTone::Bad,
    }
}

/// What the "yes" does: starts the job of each package (or the removal), after the question went away.
fn confirm(shared: &Shared) {
    let Some(p) = locked(shared).take_preview() else {
        return;
    };
    match p.action {
        PreviewAction::Update if p.names.len() > 1 => start_all_updates(shared),
        PreviewAction::Install | PreviewAction::Update => {
            for name in &p.names {
                start_job(shared, name);
            }
        }
        PreviewAction::Remove => {
            if let Some(name) = p.names.first() {
                locked(shared).ask_uninstall(name);
                start_uninstall(shared);
            }
        }
    }
    rebuild();
}

/// What the person must read before saying yes: a need nobody offers (red), a need with a choice (amber). Own lines above the steps.
fn warnings(p: &crate::model::Preview) -> Div {
    let mut col = div().flex_col().gap(2.0);
    for wn in &p.warnings {
        let tone = if wn.missing {
            BadgeTone::Bad
        } else {
            BadgeTone::Warn
        };
        col = col.child(
            div()
                .flex_row()
                .items_center()
                .gap(3.0)
                .child(badge(
                    if wn.missing {
                        w::WARN_MISSING
                    } else {
                        w::WARN_CHOICE
                    },
                    tone,
                ))
                .child(div().flex_grow().min_w(0.0).child(text(wn.text.clone()))),
        );
    }
    col
}

/// The panel, when a question waits.
pub(super) fn preview(model: &StoreModel, shared: &Shared) -> Option<Div> {
    let p: &Preview = model.preview()?;
    let mut steps = div().flex_col().gap(2.0);
    for (i, s) in p.steps.iter().enumerate() {
        steps = steps.child(
            div()
                .flex_row()
                .items_center()
                .gap(3.0)
                .child(
                    div()
                        .w(24.0)
                        .flex_shrink_0()
                        .child(label_dim(&format!("{:02}", i + 1))),
                )
                .child(
                    div()
                        .w(96.0)
                        .flex_shrink_0()
                        .child(badge(s.risk.word(), risk_tone(s.risk))),
                )
                .child(div().flex_grow().min_w(0.0).child(text(s.text.clone())))
                .child(
                    div()
                        .w(80.0)
                        .flex_shrink_0()
                        .child(label_dim(s.actor.word())),
                ),
        );
    }
    let (label, look) = (p.confirm_label(), Look::Fill);
    let mut extra = div().flex_col().gap(1.0);
    if !p.facts.is_empty() {
        extra = extra.child(label_dim(&p.facts));
    }
    if let Some(notice) = &p.notice {
        extra = extra.child(text(notice.clone()).bold());
    }
    let (yes, no) = (Arc::clone(shared), Arc::clone(shared));
    Some(
        div()
            .class("craft-panel")
            .class("craft-preview")
            .flex_col()
            .gap(3.5)
            .p(5.0)
            .child(text(p.title.clone()).bold().size(16.0))
            .child(warnings(p))
            .child(extra)
            .child(steps)
            .child(label_dim(&p.risk_line()))
            .child(
                div()
                    .flex_row()
                    .gap(2.0)
                    .child(button(
                        "preview-yes",
                        label,
                        look,
                        true,
                        Arc::new(move || confirm(&yes)),
                    ))
                    .child(button(
                        "preview-no",
                        w::BUTTON_NOT_NOW,
                        Look::Line,
                        true,
                        Arc::new(move || {
                            locked(&no).cancel_preview();
                            rebuild();
                        }),
                    )),
            ),
    )
}
