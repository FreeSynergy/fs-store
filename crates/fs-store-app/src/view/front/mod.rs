//! Pattern: Facade (the storefront, variant B: a bar with the layout switch, the filter tabs with sort and search, the
//! navigation by kind at the left, the start page, the list as a grid or a table, the page of one package, and the
//! question before any work). Split by topic: `bar`, `nav`, `list`, `page`, `preview`, `widgets`.

mod bar;
mod layout;
mod list;
mod nav;
mod page;
mod preview;
mod styles;
mod table;
mod tile;

pub(super) use bar::{BAR_H, top_bar};
pub(super) use layout::{search_field, storefront};
pub(super) use styles::STYLES;
