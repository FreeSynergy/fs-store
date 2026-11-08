//! Pattern: Adapter (`AppStream` collection XML into our package offers).
//! Format: freedesktop `AppStream` "collection metadata" (XML). Our extra fields live in `<custom>` values with the
//! `fs:` prefix, which is how `AppStream` allows extensions; no format of our own (see `DESIGN.md`).
#![deny(missing_docs)]

mod catalog;

pub use catalog::{Catalog, CatalogEntry, CatalogError, Skipped, parse_catalog};
