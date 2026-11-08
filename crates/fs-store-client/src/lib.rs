//! Pattern: Facade (one question: "is there an update for me?"). Decision (provisional, 2026-11-08): programs read the
//! result of the store's background check (`updates.toml`, written by `fs-store background`) instead of calling the
//! server `fs-store serve`: no process has to run, nothing is downloaded, and the one place that checks and downloads
//! stays the store. When `fs-store serve` can know its callers (ADR 0054) the same question can go through `store.capnp`.
//!
//! A program shows the hint, for example "A new version of this program is available (0.0.2 -> 0.0.3). Open the
//! store to update." and never downloads by itself. The person can switch hints off in the store (`notify off`).
#![deny(missing_docs)]

mod hint;

pub use hint::{UpdateHint, store_root, update_hint, update_hint_in};
