//! Pattern: Adapter (the question before any work in variant C: the fixed steps in plain words with their risk and actor, the highest
//! risk, the two answers). The steps are those the store core really runs.

use std::sync::Arc;

use fs_components::blinc::cn::components::button::ButtonVariant;
use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;

use crate::model::{PreviewAction, Risk, StoreModel};
use crate::view::job::{start_all_updates, start_job, start_uninstall};
use crate::view::shell::{Shared, button, locked, rebuild};
use crate::words_front as w;

fn risk_class(risk: Risk) -> &'static str {
    match risk {
        Risk::Look | Risk::Open => "fs-c-sub",
        Risk::Start => "fs-c-info",
        Risk::Install => "fs-c-warn",
        Risk::System => "fs-c-bad",
    }
}

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
    let p = model.preview()?;
    let mut steps = div().flex_col().gap(2.0);
    for (i, s) in p.steps.iter().enumerate() {
        steps = steps.child(
            div()
                .flex_row()
                .gap(3.0)
                .child(
                    div()
                        .w(18.0)
                        .flex_shrink_0()
                        .child(text(format!("{}.", i + 1)).class("fs-c-sub")),
                )
                .child(
                    div()
                        .w(80.0)
                        .flex_shrink_0()
                        .child(text(s.risk.word()).class(risk_class(s.risk))),
                )
                .child(div().flex_grow().min_w(0.0).child(text(s.text.clone())))
                .child(
                    div()
                        .w(70.0)
                        .flex_shrink_0()
                        .child(text(s.actor.word()).class("fs-c-sub")),
                ),
        );
    }
    let label = match p.action {
        PreviewAction::Install => w::BUTTON_DO_INSTALL,
        PreviewAction::Update => w::BUTTON_DO_UPDATE,
        PreviewAction::Remove => w::BUTTON_DO_REMOVE,
    };
    let (yes, no) = (Arc::clone(shared), Arc::clone(shared));
    Some(
        div()
            .class("fs-c-card")
            .flex_col()
            .gap(3.0)
            .child(text(p.title.clone()).bold())
            .child(steps)
            .child(text(p.risk_line()).class("fs-c-sub"))
            .child(
                div()
                    .flex_row()
                    .gap(2.0)
                    .child(
                        button("c-preview-yes", label, ButtonVariant::Primary, true)
                            .on_click(move |_| confirm(&yes)),
                    )
                    .child(
                        button(
                            "c-preview-no",
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
