//! Pattern: Adapter (`AppStream` collection XML into our package offers).
//! Format: freedesktop `AppStream` "collection metadata" (XML). Our extra fields live in `<custom>` values with the
//! `fs:` prefix, which is how `AppStream` allows extensions; no format of our own (see `DESIGN.md`).
#![deny(missing_docs)]

mod catalog;
mod details;
mod entry;
mod fields;
mod lang;
mod more;
mod title;

pub use catalog::{
    Catalog, CatalogEntry, CatalogError, Icon, RpmFile, Skipped, parse_catalog, parse_catalog_in,
};
pub use entry::{COMING_SOON, NEEDS_STORE, NOT_SUPPORTED, UNREADABLE_MIN_STORE};
