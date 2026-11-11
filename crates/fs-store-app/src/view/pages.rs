//! Pattern: none (a plain text page: About).

use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;

use crate::model::StoreModel;

pub(super) fn about() -> Div {
    let mut page = div().flex_col().class("fs-store-empty");
    for line in StoreModel::about_lines() {
        page = page.child(text(line));
    }
    page
}
