//! Pattern: Adapter (view-model to Blinc elements) on `fs-components-app`.
//!
//! Split by topic: `shell` (the window, its surface and the shared model), `job` (the job bar and starting a
//! job), `sources`, `available` and `installed` (the three screens).

mod available;
mod installed;
mod job;
mod menu;
mod pages;
mod shell;
mod sources;

pub use shell::{Shared, store_app};
