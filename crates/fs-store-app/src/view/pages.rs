//! Pattern: none (two plain text pages: Settings and About).

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::model::StoreModel;
use crate::words;

pub(super) fn settings() -> Div {
    div()
        .class("fs-store-empty")
        .child(text(words::SETTINGS_TEXT))
}

pub(super) fn about() -> Div {
    let mut page = div().flex_col().class("fs-store-empty");
    for line in StoreModel::about_lines() {
        page = page.child(text(line));
    }
    page
}
