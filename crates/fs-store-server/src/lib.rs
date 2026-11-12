//! `fs-store-server`: the contract `store.capnp` over a Unix socket. Programs ask the store whether an
//! update exists for them; they never download by themselves, the store is the one source of truth.
//! It only translates: the store's own crates do the work.
//!
//! Interface status: draft (ADR 0045). First slice: reads only, see `DESIGN.md`.
//!
//! Pattern: Facade — this file only declares modules and re-exports.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
// The generated `Server` trait is async for every method, also those that never wait.
#![allow(clippy::missing_errors_doc, clippy::unused_async_trait_impl)]

mod capabilities;
mod catalogs;
mod local;
mod program;
mod service;
mod view;
mod wire;

pub use local::LocalStore;
pub use program::serve_command;
pub use service::StoreServer;
pub use view::{PackageInfo, SkippedInfo, SourceInfo, StoreView};

#[cfg(test)]
mod wire_test;
