//! Pattern: Adapter (the strip of messages at the top, each with a close button, and the page of the history). The strip draws
//! what the model says is due; a timer asks for a new drawing when the next message runs out, so it goes away by itself.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;

use crate::model::{Notice, Severity, StoreModel};
use crate::words_lang::Lang;
use crate::words_msg as m;

use super::shell::{Shared, locked, rebuild};

/// Now, in Unix seconds (the model is handed the time, so a test can set it).
pub(super) fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// The expiry a timer is waiting for already (0: none).
static WAITING_FOR: AtomicU64 = AtomicU64::new(0);

/// Draws again when `expiry` is reached, once per expiry.
fn draw_again_at(expiry: u64) {
    if WAITING_FOR.swap(expiry, Ordering::SeqCst) == expiry {
        return;
    }
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(
            expiry.saturating_sub(now()) + 1,
        ));
        rebuild();
    });
}

fn line(n: &Notice, shared: &Shared) -> Div {
    let id = n.id;
    let shared = Arc::clone(shared);
    let class = match n.severity {
        Severity::Info => "fs-store-hint",
        Severity::Error => "fs-store-problem",
    };
    div()
        .class(class)
        .flex_row()
        .items_center()
        .gap(3.0)
        .child(div().flex_grow().min_w(0.0).child(text(n.text.clone())))
        .child(
            div()
                .id(format!("close-notice-{id}"))
                .attr("tabindex", "0")
                .attr("title", m::close_tip(Lang::current()))
                .px(2.0)
                .child(text(m::CLOSE_MARK).size(16.0))
                .on_click(move |_| {
                    locked(&shared).close_notice(id);
                    rebuild();
                }),
        )
}

/// The strip: one line per message that is due. Call [`StoreModel::sync_notices`] first.
pub(super) fn strip(model: &StoreModel, shared: &Shared) -> Div {
    let at = now();
    let mut col = div().flex_col().gap(2.0);
    for n in model.visible_notices(at) {
        col = col.child(line(&n, shared));
    }
    if let Some(expiry) = model.next_notice_expiry(at) {
        draw_again_at(expiry);
    }
    col
}

/// The page of the history: every message of the session, newest first.
pub(super) fn history(model: &StoreModel) -> Div {
    let mut page = div().flex_col().gap(2.0).class("fs-store-empty").child(
        text(m::history_menu(Lang::current()))
            .bold()
            .size(16.0),
    );
    let all = model.notice_history();
    if all.is_empty() {
        return page.child(text(m::history_empty(Lang::current())));
    }
    for n in all.iter().rev() {
        let when = crate::model::format_time(n.since);
        page = page.child(text(format!("{when}  {}", n.text)));
    }
    page
}
