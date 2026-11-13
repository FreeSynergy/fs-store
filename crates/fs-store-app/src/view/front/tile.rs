//! Pattern: Adapter (one tile of the grid in the Craft look: a numbered box and two links in the head, then the logo with the
//! two-tone name, the summary and the signature level, the state badge, the line with date and size, and the button; while a job
//! runs the button gives way to the progress). Tiles touch and share their lines; the grid gives each its width.

use std::sync::Arc;

use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_components_styles::styles::craft_sheet::caps;
use fs_components_widgets::Glyph;
use fs_components_widgets::craft::{
    BadgeTone, Look, TILE_PADDING, accent, accent_text, badge, blinc_colour, button, cell_button,
    link, progress, tile as tile_frame, tile_head, with_tip,
};

use crate::launcher::Launcher;
use crate::model::{Card, CardState};
use crate::view::card::logo;
use crate::view::more::more_menu;
use crate::view::shell::{Shared, locked, rebuild};
use crate::words;
use crate::words_front as w;

/// The links in the head of a tile: the project page (when there is one) and the page of the package.
fn links(card: &Card, shared: &Shared, launcher: &Arc<dyn Launcher>) -> Vec<Div> {
    let mut out = Vec::new();
    if !card.homepage.is_empty() {
        let (shared, launcher, name) =
            (Arc::clone(shared), Arc::clone(launcher), card.name.clone());
        out.push(link(
            &format!("site-{}", card.name),
            w::LINK_WEBSITE,
            Arc::new(move || {
                locked(&shared).open_repository(&name, launcher.as_ref());
                rebuild();
            }),
        ));
    }
    let (shared, name) = (Arc::clone(shared), card.name.clone());
    out.push(link(
        &format!("details-{}", card.name),
        w::LINK_DETAILS,
        Arc::new(move || {
            locked(&shared).open_page(&name);
            rebuild();
        }),
    ));
    out
}

/// The name in two tones: what comes before the first dot in white, the rest in the accent of the package.
fn title(card: &Card) -> Div {
    let (first, second) = match card.title.split_once('.') {
        Some((a, b)) => (format!("{a}."), b.to_owned()),
        None => (card.title.clone(), String::new()),
    };
    let mut row = div()
        .flex_row()
        .min_w(0.0)
        .child(text(first).size(18.0).bold().no_wrap());
    if !second.is_empty() {
        row = row.child(
            text(second)
                .size(18.0)
                .bold()
                .color(blinc_colour(accent_text(&card.name)))
                .no_wrap(),
        );
    }
    row
}

/// About how many characters of the summary fit beside the logo in a tile `width` pixels wide (a little less than the room, so the
/// ellipsis closes the text and the clip never cuts it).
fn summary_room(width: f32) -> usize {
    let room = (width - TILE_PADDING - 56.0 - 14.0).max(60.0);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let chars = (room / 7.2) as usize;
    chars
}

fn identity(card: &Card, width: f32) -> Div {
    let full = crate::model::summary(&card.description, usize::MAX);
    let summary = crate::model::summary(&card.description, summary_room(width));
    div().flex_row().gap(3.5).child(logo(card)).child(
        div()
            .flex_col()
            .gap(0.75)
            .flex_grow()
            .min_w(0.0)
            .overflow_clip()
            .child(title(card))
            .child(summary_line(&card.name, &summary, &full))
            .child(text(caps(card.trust)).class("craft-caps-dim").no_wrap()),
    )
}

/// The summary line; when it was cut the whole sentence is the tooltip, and the page has the whole description.
fn summary_line(name: &str, shown: &str, full: &str) -> Div {
    let shown = if shown.is_empty() { w::UNKNOWN } else { shown };
    if shown == full || full.is_empty() {
        return div().child(text(shown.to_owned()).class("craft-sub").no_wrap());
    }
    with_tip(&format!("sum-{name}"), shown, full)
}

fn state_tone(state: CardState) -> BadgeTone {
    match state {
        CardState::NotInstalled => BadgeTone::Info,
        CardState::Installed => BadgeTone::Ok,
        CardState::UpdateAvailable => BadgeTone::Warn,
    }
}

/// The button of a tile: Install, Update to x, or Open; and the three dots.
fn actions(
    card: &Card,
    busy: bool,
    width: f32,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
) -> Div {
    let name = card.name.clone();
    let main = div().flex_grow().flex_col();
    let (label, look): (String, Look) = match card.state {
        CardState::NotInstalled => (words::BUTTON_INSTALL.to_owned(), Look::Fill),
        CardState::UpdateAvailable => (
            words::fill(
                w::BUTTON_UPDATE_TO,
                &[(
                    "version",
                    &card.offered.map(|v| v.to_string()).unwrap_or_default(),
                )],
            ),
            Look::Fill,
        ),
        CardState::Installed => (words::BUTTON_OPEN.to_owned(), Look::Line),
    };
    let (shared2, launcher2, name2, state) = (
        Arc::clone(shared),
        Arc::clone(launcher),
        name.clone(),
        card.state,
    );
    let go: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        if state == CardState::Installed {
            locked(&shared2).open(&name2, None, launcher2.as_ref());
        } else {
            locked(&shared2).ask_install(&name2);
        }
        rebuild();
    });
    let _ = width;
    div()
        .flex_row()
        .gap(2.0)
        .child(main.child(button(
            &format!("do-{name}"),
            &label,
            look,
            match state {
                CardState::Installed => card.open_blocked.is_none(),
                _ => !busy && card.working.is_none() && card.install_blocked.is_none(),
            },
            go,
        )))
        .child(
            div()
                .flex_shrink_0()
                .child(more_menu(card, shared, launcher)),
        )
}

/// Cancel: pressable while the work can be stopped with nothing left half done (the model says), else locked with its reason.
fn cancel_button(card: &Card, shared: &Shared) -> Div {
    let (shared, name) = (Arc::clone(shared), card.name.clone());
    cell_button(
        &format!("cancel-{}", card.name),
        Glyph::Close,
        if card.can_cancel {
            w::CANCEL_TIP
        } else {
            w::CANCEL_LOCKED
        },
        card.can_cancel,
        Arc::new(move || {
            locked(&shared).cancel_job(&name);
            rebuild();
        }),
    )
}

/// While a job runs: its phase, the bar, and pause and cancel (locked: the store cannot stop a job yet).
fn running(card: &Card, shared: &Shared, share: f32, line: &str, width: f32) -> Div {
    let a = accent(&card.name);
    div()
        .flex_col()
        .gap(2.0)
        .child(text(line.to_owned()).class("craft-sub"))
        .child(
            div()
                .flex_row()
                .items_center()
                .gap(2.0)
                .child(progress(share, width - 2.0 * 36.0 - 16.0, a))
                .child(cell_button(
                    &format!("pause-{}", card.name),
                    Glyph::Details,
                    w::PAUSE_LOCKED,
                    false,
                    Arc::new(|| {}),
                ))
                .child(cancel_button(card, shared)),
        )
}

/// The tile of `card`, `number` in the list, `width` pixels wide.
pub(super) fn tile(
    card: &Card,
    number: usize,
    width: f32,
    busy: bool,
    shared: &Shared,
    launcher: &Arc<dyn Launcher>,
) -> Div {
    let inner = width - TILE_PADDING;
    let mut body = div().flex_col().gap(3.5).child(identity(card, width));
    let progress_view = card.progress();
    let state_badge = match card.working {
        Some(_) => badge(card.working_word(), BadgeTone::Info),
        None => badge(card.state_word(), state_tone(card.state)),
    };
    body = body.child(div().flex_row().child(state_badge));
    body = body.child(
        div()
            .flex_col()
            .gap(1.0)
            .child(text(card.version_line_long()).class("craft-sub").no_wrap())
            .child(text(card.meta_line()).class("craft-sub").no_wrap()),
    );
    if let Some(why) = &card.failure {
        body = body.child(text(why.clone()).class("craft-sub"));
    }
    if let Some((share, line)) = progress_view {
        body = body.child(running(card, shared, share, &line, inner));
    } else {
        body = body.child(actions(card, busy, inner, shared, launcher));
        if let Some(why) = card.open_blocked.filter(|_| card.installed.is_some()) {
            body = body.child(text(why).class("craft-sub"));
        }
        if let Some(why) = &card.install_blocked {
            body = body.child(text(why).class("craft-sub"));
        }
    }
    tile_frame(
        width,
        tile_head(number, accent(&card.name), links(card, shared, launcher)),
        body,
    )
}
