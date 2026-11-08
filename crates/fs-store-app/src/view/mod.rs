//! Pattern: Adapter (view-model to Blinc elements) on `fs-components-app`.
//!
//! Split by topic: `shell` (the window, its surface and the shared model), `job` (the job bar and starting a
//! job), `cards` and `card` (the one page), `more` (the three-dot menu), `admin` (the settings page: update choices, timer, programs, sources).

mod admin;
mod card;
mod cards;
mod check;
mod job;
mod menu;
mod more;
mod pages;
mod settings;
mod shell;
mod sources;

pub use check::start_check;
pub use shell::{Shared, store_app};
