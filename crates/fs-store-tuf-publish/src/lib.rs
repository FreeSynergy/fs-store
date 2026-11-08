//! Pattern: Adapter (`tough`'s editor behind two functions). Publishing side of TUF (ADR 0062); the reading side is
//! `fs-store-tuf`. Given a directory with `catalog.xml`, the trusted `root.json` and the key files `targets.pk8`,
//! `snapshot.pk8`, `timestamp.pk8` (Ed25519, PKCS#8 DER), [`publish`] writes `tuf/metadata/*.json` and
//! `tuf/targets/catalog.xml`. Versions only go up: they are read from the metadata already there. Consistent snapshots are off.
#![deny(missing_docs)]

mod keys;
mod publish;

pub use keys::generate_keys;
pub use publish::*;
