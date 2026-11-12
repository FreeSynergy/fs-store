//! Pattern: Adapter (the table layout in the Craft look: a head of dim capitals, rows with a line above, a button at the end).

use std::sync::Arc;

use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;

use crate::launcher::Launcher;
use crate::model::{CardState, NeedsState, StoreModel, TableRow};
use crate::view::card::logo_small;
use crate::view::shell::{Shared, locked, rebuild};
use crate::words;
use crate::words_front as w;

use fs_components_widgets::craft::{BadgeTone, Look, badge, button, label_dim};

fn needs_tone(state: NeedsState) -> BadgeTone {
    match state {
        NeedsState::None | NeedsState::Met => BadgeTone::Ok,
        NeedsState::Choice => BadgeTone::Warn,
        NeedsState::Missing => BadgeTone::Bad,
    }
}

/// Width of the column of buttons.
const ACTION_W: f32 = 132.0;

const COLS: [(&str, f32); 6] = [
    (w::COL_INSTALLED, 76.0),
    (w::COL_LATEST, 76.0),
    (w::COL_RELEASED, 96.0),
    (w::COL_SIZE, 72.0),
    (w::COL_SIGNATURE, 124.0),
    (w::COL_CAPABILITIES, 74.0),
];

fn cell(width: f32, child: Div) -> Div {
    div().w(width).flex_shrink_0().child(child)
}

fn head() -> Div {
    let mut row = div()
        .class("craft-thead")
        .flex_row()
        .items_center()
        .gap(2.0)
        .child(
            div()
                .flex_grow()
                .min_w(0.0)
                .child(label_dim(w::COL_PACKAGE)),
        );
    for (name, width) in COLS {
        row = row.child(cell(width, div().child(label_dim(name))));
    }
    row.child(div().w(ACTION_W).flex_shrink_0())
}

fn action(row: &TableRow, busy: bool, shared: &Shared, launcher: &Arc<dyn Launcher>) -> Div {
    let name = row.card.name.clone();
    let state = row.card.state;
    let (label, look) = match state {
        CardState::NotInstalled => (words::BUTTON_INSTALL.to_owned(), Look::Fill),
        CardState::UpdateAvailable => (
            words::fill(w::BUTTON_UPDATE_TO, &[("version", &row.latest)]),
            Look::Fill,
        ),
        CardState::Installed => (words::BUTTON_OPEN.to_owned(), Look::Line),
    };
    let (shared, launcher, id) = (
        Arc::clone(shared),
        Arc::clone(launcher),
        format!("do-{name}"),
    );
    div()
        .w(ACTION_W)
        .flex_shrink_0()
        .flex_row()
        .child(div().flex_grow().flex_col().child(button(
            &id,
            &label,
            look,
            match state {
                CardState::Installed => row.card.open_blocked.is_none(),
                _ => !busy && row.card.working.is_none(),
            },
            Arc::new(move || {
                if state == CardState::Installed {
                    locked(&shared).open(&name, None, launcher.as_ref());
                } else {
                    locked(&shared).ask_install(&name);
                }
                rebuild();
            }),
        )))
}

fn table_row(row: &TableRow, busy: bool, shared: &Shared, launcher: &Arc<dyn Launcher>) -> Div {
    let name = row.card.name.clone();
    let open = Arc::clone(shared);
    let who = div()
        .id(format!("row-{name}"))
        .flex_row()
        .items_center()
        .gap(3.0)
        .flex_grow()
        .min_w(0.0)
        .cursor_pointer()
        .attr("tabindex", "0")
        .child(logo_small(&row.card))
        .child(
            div()
                .flex_col()
                .min_w(0.0)
                .overflow_clip()
                .child(text(row.card.title.clone()).bold().no_wrap())
                .child(
                    text(if row.summary.is_empty() {
                        row.kind.clone()
                    } else {
                        format!("{} - {}", row.kind, row.summary)
                    })
                    .class("craft-sub")
                    .no_wrap(),
                ),
        )
        .on_click(move |_| {
            locked(&open).open_page(&name);
            rebuild();
        });
    let plain = |t: &str| div().child(text(t.to_owned()).class("craft-mono-text").no_wrap());
    div()
        .class("craft-trow")
        .flex_row()
        .items_center()
        .gap(2.0)
        .child(who)
        .child(cell(COLS[0].1, plain(&row.installed)))
        .child(cell(COLS[1].1, plain(&row.latest)))
        .child(cell(COLS[2].1, plain(&row.released)))
        .child(cell(COLS[3].1, plain(&row.size)))
        .child(cell(COLS[4].1, plain(&row.signature)))
        .child(cell(
            COLS[5].1,
            div().child(badge(&row.needs, needs_tone(row.needs_state))),
        ))
        .child(action(row, busy, shared, launcher))
}

pub(super) fn table(model: &StoreModel, shared: &Shared, launcher: &Arc<dyn Launcher>) -> Div {
    let mut table = div().class("craft-box").flex_col().child(head());
    for row in model.table_rows() {
        table = table.child(table_row(&row, model.is_removing(), shared, launcher));
    }
    table
}
