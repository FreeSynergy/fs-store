//! Pattern: Adapter (view-model to Blinc elements) on `fs-components-app`.
//!
//! Split by topic: `shell` (the window, its surface and the shared model), `front` (the storefront, variant B: bar, navigation,
//! start page, grid and table, package page, the question before any work), `job` (the job bar and starting a job), `card`
//! (one card), `more` (the three-dot menu), `admin` (the settings page: update choices, timer, programs, sources).

mod admin;
mod card;
mod check;
mod front;
mod job;
mod menu;
mod more;
mod pages;
mod settings;
mod shell;
mod sources;

pub use check::start_check;
pub use shell::{Shared, store_app};
