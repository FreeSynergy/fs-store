//! Pattern: Adapter (view-model to Blinc elements) on `fs-components-app`: three screens (Sources, Available,
//! Installed), each a [`Surface`] over the same shared [`StoreModel`]. A click changes the model and asks for
//! a new build.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_cn::components::button::{ButtonBuilder, ButtonVariant};
use blinc_core::BlincContextState;
use blinc_layout::div::div;
use blinc_layout::prelude::text;
use fs_components_app::{AppError, AppWidget, Element, Screen, Surface};

use crate::model::{StoreModel, View};
use crate::words;

use super::available::available;
use super::installed::installed;
use super::job::job_bar;
use super::sources::sources;

/// The model, shared between the window and the worker of a job.
pub type Shared = Arc<Mutex<StoreModel>>;

pub(super) fn locked(model: &Shared) -> MutexGuard<'_, StoreModel> {
    model.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Draw again. Safe from any thread: it only sets a flag.
pub(super) fn rebuild() {
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

pub(super) fn button(
    key: &str,
    label: &str,
    variant: ButtonVariant,
    enabled: bool,
) -> ButtonBuilder {
    ButtonBuilder::with_key(key, label.to_owned())
        .variant(variant)
        .disabled(!enabled)
}
