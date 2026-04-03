// wizard/mod.rs — Multi-step install wizard (fs-store-app).
#![allow(dead_code)] // infrastructure — will be wired up when Dioxus is replaced (G2)
                     //
                     // Steps:
                     //   select.rs   — package selection list
                     //   confirm.rs  — details + env-var input
                     //   progress.rs — live install progress
                     //   done.rs     — success / failure result

pub mod confirm;
pub mod done;
pub mod engine_select;
pub mod progress;
pub mod select;
