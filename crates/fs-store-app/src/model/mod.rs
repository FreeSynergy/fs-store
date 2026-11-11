//! Pattern: View-model (state plus commands, no GUI types). The window only draws this and calls its commands.
//!
//! Split by topic: `types` (plain data), `cards` and `cards_ui` (the one page of cards), `store` (the state and reading the backend), `sources` (commands on
//! the source list), `jobs` (the job of one package), `text` (the sentences the window shows).

mod admin;
mod cards;
mod cards_ui;
mod fetch;
mod job_end;
mod jobs;
mod menu;
mod open;
mod refresh;
mod sources;
mod store;
mod text;
mod titles;
mod types;
mod update_text;
mod updates;

pub use cards::{
    Buttons, Card, CardState, OpenWidth, Placeholder, Working, buttons_for, columns, placeholder,
};
pub use cards_ui::CardsUi;
pub use fetch::{Fetched, fetch};
pub use jobs::{JobState, JobView, Ticket};
pub use menu::{Command, Effect, help_url, is_help_key, is_quit_key};
pub use refresh::{ReadTicket, RefreshState};
pub use store::StoreModel;
pub use titles::UpdateRow;
pub use types::{Action, AvailableRow, Problem, View};
pub use update_text::{check_label, format_time, install_label};
pub use updates::{CheckState, check_due};
