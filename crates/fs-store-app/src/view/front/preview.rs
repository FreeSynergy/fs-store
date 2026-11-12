//! Pattern: Adapter (the question before any work: the fixed steps in plain words, each with how far it reaches and who does
//! it, the highest risk, and the two answers). The steps are those the store core really runs.

use std::sync::Arc;

use fs_components::blinc::cn::components::button::ButtonVariant;
use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;

use crate::model::{Preview, PreviewAction, Risk, StoreModel};
use crate::view::job::{start_all_updates, start_job, start_uninstall};
use crate::view::shell::{Shared, button, locked, rebuild};
use crate::words_front as w;

use super::widgets::badge;

fn risk_tone(risk: Risk) -> &'static str {
    match risk {
        Risk::Look | Risk::Open => "fs-store-badge-quiet",
        Risk::Start => "fs-store-badge-info",
        Risk::Install => "fs-store-badge-warn",
        Risk::System => "fs-store-badge-bad",
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
                        .w(22.0)
                        .flex_shrink_0()
                        .child(text(format!("{}.", i + 1)).class("fs-store-sub")),
                )
                .child(
                    div()
                        .w(92.0)
                        .flex_shrink_0()
                        .child(badge(s.risk.word(), risk_tone(s.risk))),
                )
                .child(div().flex_grow().min_w(0.0).child(text(s.text.clone())))
                .child(
                    div()
                        .w(70.0)
                        .flex_shrink_0()
                        .child(text(s.actor.word()).class("fs-store-sub").no_wrap()),
                ),
        );
    }
    let (label, variant) = match p.action {
        PreviewAction::Install => (w::BUTTON_DO_INSTALL, ButtonVariant::Primary),
        PreviewAction::Update => (w::BUTTON_DO_UPDATE, ButtonVariant::Primary),
        PreviewAction::Remove => (w::BUTTON_DO_REMOVE, ButtonVariant::Destructive),
    };
    let (yes, no) = (Arc::clone(shared), Arc::clone(shared));
    Some(
        div()
            .class("fs-store-preview")
            .flex_col()
            .gap(3.0)
            .child(text(p.title.clone()).bold().size(16.0))
            .child(steps)
            .child(text(p.risk_line()).class("fs-store-sub"))
            .child(
                div()
                    .flex_row()
                    .gap(2.0)
                    .child(
                        button("preview-yes", label, variant, true)
                            .on_click(move |_| confirm(&yes)),
                    )
                    .child(
                        button(
                            "preview-no",
                            w::BUTTON_NOT_NOW,
                            ButtonVariant::Outline,
                            true,
                        )
                        .on_click(move |_| {
                            locked(&no).cancel_preview();
                            rebuild();
                        }),
                    ),
            ),
    )
}
