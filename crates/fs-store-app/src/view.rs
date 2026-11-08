//! Pattern: Adapter (view-model to Blinc elements) on `fs-components-app`: three screens (Sources, Available,
//! Installed), each a [`Surface`] over the same shared [`StoreModel`]. A click changes the model and asks for
//! a new build; a job runs on a worker thread and reports each phase the same way.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_cn::components::button::{ButtonBuilder, ButtonVariant};
use blinc_core::BlincContextState;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use fs_components_app::{AppError, AppWidget, Element, Screen, Surface};
use fs_components_primitives::SearchField;

use crate::backend::{JobError, Phase};
use crate::model::{Action, AvailableRow, StoreModel, View};
use crate::words;

/// The model, shared between the window and the worker of a job.
pub type Shared = Arc<Mutex<StoreModel>>;

fn locked(model: &Shared) -> MutexGuard<'_, StoreModel> {
    model.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Draw again. Safe from any thread: it only sets a flag.
fn rebuild() {
    if BlincContextState::is_initialized() {
        BlincContextState::get().request_rebuild();
    }
}

/// The window: three screens over `model`.
///
/// # Errors
/// [`AppError`] if the shell cannot be made.
pub fn store_app(model: &Shared) -> Result<AppWidget, AppError> {
    let screen = |id: &str, label: &str, view| {
        Screen::new(
            id,
            label,
            Arc::new(StoreSurface {
                model: Arc::clone(model),
                view,
            }),
        )
    };
    AppWidget::new(
        words::TITLE,
        vec![
            screen("available", words::VIEW_AVAILABLE, View::Available),
            screen("installed", words::VIEW_INSTALLED, View::Installed),
            screen("sources", words::VIEW_SOURCES, View::Sources),
        ],
    )
}

/// One of the three views as a surface.
struct StoreSurface {
    model: Shared,
    view: View,
}

impl Surface for StoreSurface {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        let mut model = locked(&self.model);
        model.show(self.view);
        let mut page = div()
            .class("fs-store-page")
            .flex_col()
            .w(width)
            .h(height)
            .child(job_bar(&model));
        for line in model.problem_lines() {
            page = page.child(div().class("fs-store-problem").child(text(line)));
        }
        let body = match self.view {
            View::Sources => sources(&model, &self.model),
            View::Available => available(&model, &self.model),
            View::Installed => installed(&model),
        };
        Box::new(page.child(body))
    }

    fn styles(&self) -> String {
        STYLES.to_owned()
    }
}

const STYLES: &str = r"
.fs-store-page { display: flex; flex-direction: column; gap: var(--space-2); padding: var(--space-3); overflow: scroll; }
.fs-store-job { display: flex; flex-direction: column; gap: var(--space-1); padding: var(--space-2); border-width: 1px; border-color: var(--border); background: var(--surface); }
.fs-store-phases { display: flex; flex-direction: row; gap: var(--space-2); }
.fs-store-phase { color: var(--text-secondary); font-size: var(--text-sm); }
.fs-store-phase-now { color: var(--text-primary); font-weight: 600; }
.fs-store-problem { padding: var(--space-2); background: var(--error-bg); color: var(--text-primary); }
.fs-store-row { display: flex; flex-direction: row; align-items: center; gap: var(--space-3); padding: var(--space-2); border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-store-cell { flex-grow: 1; min-width: 0; }
.fs-store-sub { color: var(--text-secondary); font-size: var(--text-sm); }
.fs-store-empty { padding: var(--space-4); color: var(--text-secondary); }
.fs-store-add { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); }
";

/// The six phases in a row; the one the job is in stands out. After a failure, `Failed` is told too.
fn job_bar(model: &StoreModel) -> Div {
    let mut bar = div().class("fs-store-job").child(text(model.job_line()));
    if let Some(job) = model.job() {
        let mut marks = div().class("fs-store-phases");
        for phase in [
            Phase::Pending,
            Phase::Verified,
            Phase::Installing,
            if job.error.is_some() {
                if job.error == Some(JobError::RollbackFailed) {
                    Phase::Failed
                } else {
                    Phase::RolledBack
                }
            } else {
                Phase::Installed
            },
        ] {
            let mut mark = div().child(text(words::phase_name(phase)).class("fs-store-phase"));
            if job.phase == phase {
                mark = mark.class("fs-store-phase-now");
            }
            if job.history.contains(&phase) {
                marks = marks.child(mark);
            } else {
                marks = marks
                    .child(div().child(text(words::phase_name(phase)).class("fs-store-phase")));
            }
        }
        bar = bar.child(marks);
    }
    bar
}

fn button(key: &str, label: &str, variant: ButtonVariant, enabled: bool) -> ButtonBuilder {
    ButtonBuilder::with_key(key, label.to_owned())
        .variant(variant)
        .disabled(!enabled)
}

fn sources(model: &StoreModel, shared: &Shared) -> Div {
    let typed = Arc::clone(shared);
    let field = SearchField::new(words::ADD_PLACEHOLDER).on_change(Arc::new(move |t: &str| {
        locked(&typed).set_new_source(t);
    }));
    let add = Arc::clone(shared);
    let mut page = div().flex_col().w_full().child(
        div().class("fs-store-add").child(field.build()).child(
            button(
                "add-source",
                words::BUTTON_ADD,
                ButtonVariant::Primary,
                true,
            )
            .on_click(move |_| {
                locked(&add).add_source();
                rebuild();
            }),
        ),
    );
    if model.sources().is_empty() {
        return page.child(
            div()
                .class("fs-store-empty")
                .child(text(words::EMPTY_SOURCES)),
        );
    }
    for source in model.sources() {
        let mut name = source.id.clone();
        if source.reference {
            name = format!("{name} ({})", words::MARK_BUILT_IN);
        }
        if !source.enabled {
            name = format!("{name} ({})", words::MARK_OFF);
        }
        let (toggle_label, toggle_variant) = if source.enabled {
            (words::BUTTON_SWITCH_OFF, ButtonVariant::Outline)
        } else {
            (words::BUTTON_SWITCH_ON, ButtonVariant::Secondary)
        };
        let (toggle, remove) = (Arc::clone(shared), Arc::clone(shared));
        let (toggle_id, remove_id, now) = (source.id.clone(), source.id.clone(), source.enabled);
        let mut row = div()
            .class("fs-store-row")
            .child(
                div()
                    .class("fs-store-cell")
                    .flex_col()
                    .child(text(name))
                    .child(text(source.url.clone()).class("fs-store-sub")),
            )
            .child(
                button(
                    &format!("toggle-{}", source.id),
                    toggle_label,
                    toggle_variant,
                    true,
                )
                .on_click(move |_| {
                    locked(&toggle).set_enabled(&toggle_id, !now);
                    rebuild();
                }),
            );
        // The reference source cannot be removed: no button for it (the model refuses anyway).
        if !source.reference {
            row = row.child(
                button(
                    &format!("remove-{}", source.id),
                    words::BUTTON_REMOVE,
                    ButtonVariant::Destructive,
                    true,
                )
                .on_click(move |_| {
                    locked(&remove).remove_source(&remove_id);
                    rebuild();
                }),
            );
        }
        page = page.child(row);
    }
    page
}

fn available(model: &StoreModel, shared: &Shared) -> Div {
    let refresh = Arc::clone(shared);
    let mut page = div().flex_col().w_full().child(
        button(
            "refresh",
            words::BUTTON_REFRESH,
            ButtonVariant::Ghost,
            !model.is_busy(),
        )
        .on_click(move |_| {
            locked(&refresh).refresh();
            rebuild();
        }),
    );
    if model.available().is_empty() {
        return page.child(
            div()
                .class("fs-store-empty")
                .child(text(words::EMPTY_AVAILABLE)),
        );
    }
    for row in model.available() {
        page = page.child(available_row(row, model.is_busy(), shared));
    }
    page
}

fn available_row(row: &AvailableRow, busy: bool, shared: &Shared) -> Div {
    let package = &row.package;
    let version = package.version.to_string();
    let sub = match row.installed {
        Some(installed) => words::fill(
            words::NEWER_OFFERED,
            &[
                ("offered", version.as_str()),
                ("installed", installed.to_string().as_str()),
            ],
        ),
        None => version.clone(),
    };
    let line = format!("{} - {}", words::kind(package.kind), sub);
    let cell = div()
        .class("fs-store-cell")
        .flex_col()
        .child(text(package.name.clone()))
        .child(text(line).class("fs-store-sub"));
    let action: Div = match row.action {
        Action::UpToDate => div().child(
            text(words::fill(
                words::UP_TO_DATE,
                &[("version", version.as_str())],
            ))
            .class("fs-store-sub"),
        ),
        Action::Install | Action::Update => {
            let label = if row.action == Action::Install {
                words::BUTTON_INSTALL
            } else {
                words::BUTTON_UPDATE
            };
            let (shared, name) = (Arc::clone(shared), package.name.clone());
            div().child(
                button(
                    &format!("job-{}", package.name),
                    label,
                    ButtonVariant::Primary,
                    !busy,
                )
                .on_click(move |_| start_job(&shared, &name)),
            )
        }
    };
    div().class("fs-store-row").child(cell).child(action)
}

fn installed(model: &StoreModel) -> Div {
    let mut page = div().flex_col().w_full();
    if model.installed().is_empty() {
        return page.child(
            div()
                .class("fs-store-empty")
                .child(text(words::EMPTY_INSTALLED)),
        );
    }
    for row in model.installed() {
        page = page.child(
            div()
                .class("fs-store-row")
                .child(div().class("fs-store-cell").child(text(row.name.clone())))
                .child(text(row.version.to_string()).class("fs-store-sub")),
        );
    }
    page
}

/// Starts the job of `name` on a worker thread; every phase asks the window to draw again.
fn start_job(shared: &Shared, name: &str) {
    let (backend, kind, name) = {
        let mut model = locked(shared);
        let Some(ticket) = model.begin_job(name) else {
            drop(model);
            rebuild();
            return;
        };
        let (kind, name) = (ticket.kind, ticket.name.to_owned());
        (model.backend(), kind, name)
    };
    rebuild();
    let shared = Arc::clone(shared);
    std::thread::spawn(move || {
        let result = backend.run(kind, &name, &mut |phase| {
            locked(&shared).observe(phase);
            rebuild();
        });
        locked(&shared).finish(result);
        rebuild();
    });
}
