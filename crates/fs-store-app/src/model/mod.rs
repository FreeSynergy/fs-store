//! Pattern: View-model (state plus commands, no GUI types). The window only draws this and calls its commands.
//!
//! Split by topic: `types` (plain data), `store` (the state and reading the backend), `sources` (commands on
//! the source list), `jobs` (the job of one package), `text` (the sentences the window shows).

mod jobs;
mod menu;
mod open;
mod sources;
mod store;
mod text;
mod types;

pub use jobs::{JobView, Ticket};
pub use menu::{Command, Effect, is_quit_key};
pub use store::StoreModel;
pub use types::{Action, AvailableRow, Problem, View};
