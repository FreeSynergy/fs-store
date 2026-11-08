//! Pattern: none (a plain text page: About).

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::model::StoreModel;

pub(super) fn about() -> Div {
    let mut page = div().flex_col().class("fs-store-empty");
    for line in StoreModel::about_lines() {
        page = page.child(text(line));
    }
    page
}
