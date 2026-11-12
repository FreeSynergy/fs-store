//! FreeSynergy.Store, first window (`fs-store` opens it when started without arguments).
//!
//! Layers, from the inside: `backend` (the trait `StoreBackend`, shaped like `store.capnp`, so a Cap'n Proto
//! client can replace the in-process engine), `model` (view-model, pure Rust, no GUI types), `words` (every
//! text a person reads), `engine` (the in-process backend), `view` (Blinc on top of `fs-components-app`).
#![deny(missing_docs)]

pub mod backend;
pub mod demo;
mod demo_data;
pub mod engine;
mod engine_after;
mod engine_job;
mod engine_map;
mod engine_remove;
pub mod gui;
pub mod installed;
pub mod launcher;
pub mod model;
pub mod updates;
pub mod view;
pub mod words;
pub mod words_front;
mod words_job;
