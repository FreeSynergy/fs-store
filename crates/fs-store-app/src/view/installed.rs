//! Pattern: none (a plain list of rows; no commands on this screen).

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::model::StoreModel;
use crate::words;

pub(super) fn installed(model: &StoreModel) -> Div {
    let mut page = div().flex_col().w_full();
    if model.installed().is_empty() {
        return page.child(
            div()
                .class("fs-store-empty")
                .child(text(words::EMPTY_INSTALLED)),
        );
    }
    for row in model.installed() {
        page = page.child(
            div()
                .class("fs-store-row")
                .child(div().class("fs-store-cell").child(text(row.name.clone())))
                .child(text(row.version.to_string()).class("fs-store-sub")),
        );
    }
    page
}
