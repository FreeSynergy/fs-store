//! Pattern: Adapter (view-model to Blinc elements) on `fs-components-app`: three screens (Sources, Available,
//! Installed), each a [`Surface`] over the same shared [`StoreModel`]. A click changes the model and asks for
//! a new build.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_cn::components::button::{ButtonBuilder, ButtonVariant};
use blinc_core::BlincContextState;
use blinc_layout::div::div;
use blinc_layout::prelude::text;
use fs_components_app::{AppError, AppWidget, Element, Screen, Surface};

use crate::launcher::Launcher;

use crate::model::{StoreModel, View, is_help_key, is_quit_key};
use crate::words;

use super::admin::admin;
use super::cards::cards;
use super::job::job_list;
use super::menu::{install_menus, open_help, quit};
use super::pages::about;

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

/// Id of the one screen.
const SCREEN: &str = "store";

/// The window: one screen (so no navigation to search in) and the menu over `model`; programs are opened
/// with `launcher`. The page shown (cards, sources, settings, about) is the model's view.
///
/// # Errors
/// [`AppError`] if the shell cannot be made.
pub fn store_app(model: &Shared, launcher: &Arc<dyn Launcher>) -> Result<AppWidget, AppError> {
    let title = locked(model).window_title();
    let screen = Screen::new(
        SCREEN,
        words::TITLE,
        Arc::new(StoreSurface {
            model: Arc::clone(model),
            launcher: Arc::clone(launcher),
        }),
    );
    let app = AppWidget::new(title, vec![screen])?;
    install_menus(&app, model, launcher)?;
    Ok(app)
}

/// One of the views as a surface.
struct StoreSurface {
    model: Shared,
    launcher: Arc<dyn Launcher>,
}

impl Surface for StoreSurface {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        let mut model = locked(&self.model);
        let view = model.view();
        model.show(view);
        let mut page = div()
            .class("fs-store-page")
            .flex_col()
            .w(width)
            .h(height)
            .on_key_down({
                let shared = Arc::clone(&self.model);
                let launcher = Arc::clone(&self.launcher);
                move |ctx| {
                    if is_quit_key(ctx.key_code, ctx.ctrl) {
                        quit(&shared);
                    } else if is_help_key(ctx.key_code) {
                        open_help(&shared, &launcher);
                    }
                }
            });
        if !model.jobs().is_empty() {
            page = page.child(job_list(&model, &self.model));
        }
        if view != View::Cards {
            let back = Arc::clone(&self.model);
            page = page.child(div().child(
                button("back", words::BUTTON_BACK, ButtonVariant::Ghost, true).on_click(
                    move |_| {
                        locked(&back).show(View::Cards);
                        rebuild();
                    },
                ),
            ));
        }
        for line in model.problem_lines() {
            page = page.child(div().class("fs-store-problem").child(text(line)));
        }
        let body = match view {
            View::Cards => cards(&model, &self.model, &self.launcher, width),
            View::Sources | View::Settings => admin(&model, &self.model, view == View::Sources),
            View::About => about(),
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
.fs-card { display: flex; flex-direction: column; padding: 12px; border-width: 1px; border-color: var(--border); border-radius: 8px; background: var(--surface); }
.fs-card-rest { }
.fs-card:hover { background: var(--surface-hover); border-color: var(--text-secondary); }
.fs-card-selected { border-color: var(--primary); border-width: 2px; }
.fs-card-head { display: flex; flex-direction: row; align-items: center; gap: 12px; }
.fs-card-badge { color: var(--primary); font-size: var(--text-sm); font-weight: 600; }
.fs-card-working { padding: 8px; color: var(--text-secondary); }
.fs-wide { width: 100%; }
.fs-more { border-width: 1px; border-color: var(--border); border-radius: 6px; }
.fs-more-open { background: var(--surface-hover); }
.fs-panel { padding: 16px; border-width: 1px; border-color: var(--border); border-radius: 8px; background: var(--surface); }
.fs-store-add { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); }
.fs-store-section { display: flex; flex-direction: column; gap: var(--space-1); padding: var(--space-2) var(--space-4); color: var(--text-secondary); }
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
