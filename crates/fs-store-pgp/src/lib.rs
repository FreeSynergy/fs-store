//! Adapter: rPGP behind `fs_store_core::SignatureVerifier`.
//! Trusts only the keys it is given; the built-in release key is `PgpVerifier::builtin`.
#![deny(missing_docs)]

mod verifier;

pub use verifier::{KeyError, PgpVerifier};
