//! Pattern: Adapter (one tile of the grid in the Craft look: a numbered box and two links in the head, then the logo with the
//! two-tone name, the summary and the signature level, the state badge, the line with date and size, and the button; while a job
//! runs the button gives way to the progress). Tiles touch and share their lines; the grid gives each its width.

use std::sync::Arc;

use fs_components::blinc::core::Color;
use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_components_styles::styles::craft::{ACCENTS, ACCENTS_TEXT, accent_index};
use fs_components_styles::styles::craft_sheet::caps;
use fs_components_widgets::Glyph;

use crate::launcher::Launcher;
use crate::model::{Card, CardState};
use crate::view::card::logo;
use crate::view::more::more_menu;
use crate::view::shell::{Shared, locked, rebuild};
use crate::words;
use crate::words_front as w;

use super::widgets::{Look, badge, button, cell_button, number_box, progress};

/// The accent colour of a package as bytes: the same name has the same colour every time.
pub(super) fn accent(name: &str) -> (u8, u8, u8) {
    let c = ACCENTS[accent_index(name)];
    (c.r, c.g, c.b)
}

fn accent_text(name: &str) -> Color {
    let c = ACCENTS_TEXT[accent_index(name)];
    Color::rgb(
        f32::from(c.r) / 255.0,
        f32::from(c.g) / 255.0,
        f32::from(c.b) / 255.0,
    )
}

/// A link in the head of a tile: dim capitals with an arrow.
fn link(id: String, words: &str, act: impl Fn() + Send + Sync + 'static) -> Div {
    div()
        .id(id)
        .class("craft-link")
        .attr("tabindex", "0")
        .child(
            text(format!("{} \u{2197}", caps(words)))
                .class("craft-caps-dim")
                .no_wrap(),
        )
        .on_click(move |_| act())
}

fn head(card: &Card, number: usize, shared: &Shared, launcher: &Arc<dyn Launcher>) -> Div {
    let mut head = div()
        .class("craft-tile-head")
        .flex_row()
        .items_center()
        .gap(3.5)
        .child(number_box(number, accent(&card.name)))
        .child(div().flex_grow());
    if !card.homepage.is_empty() {
        let (shared, launcher, name) =
            (Arc::clone(shared), Arc::clone(launcher), card.name.clone());
        head = head.child(link(
            format!("site-{}", card.name),
            w::LINK_WEBSITE,
            move || {
                locked(&shared).open_repository(&name, launcher.as_ref());
                rebuild();
            },
        ));
    }
    let (shared, name) = (Arc::clone(shared), card.name.clone());
    head.child(link(
        format!("details-{}", card.name),
        w::LINK_DETAILS,
        move || {
            locked(&shared).open_page(&name);
            rebuild();
        },
    ))
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
                .color(accent_text(&card.name))
                .no_wrap(),
        );
    }
    row
}

/// About how many characters of the summary fit beside the logo in a tile `width` pixels wide (a little less than the room, so the
/// ellipsis closes the text and the clip never cuts it).
fn summary_room(width: f32) -> usize {
    let room = (width - 41.0 - 56.0 - 14.0).max(60.0);
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
    super::widgets::with_tip(&format!("sum-{name}"), shown, full)
}

fn state_tone(state: CardState) -> &'static str {
    match state {
        CardState::NotInstalled => "fs-t-info",
        CardState::Installed => "fs-t-ok",
        CardState::UpdateAvailable => "fs-t-warn",
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
            !busy || state == CardState::Installed,
            go,
        )))
        .child(
            div()
                .flex_shrink_0()
                .child(more_menu(card, shared, launcher)),
        )
}

/// While a job runs: its phase, the bar, and pause and cancel (locked: the store cannot stop a job yet).
fn running(card: &Card, share: f32, line: &str, width: f32) -> Div {
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
                .child(cell_button(
                    &format!("cancel-{}", card.name),
                    Glyph::Close,
                    w::CANCEL_LOCKED,
                    false,
                    Arc::new(|| {}),
                )),
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
    let inner = width - 41.0;
    let mut body = div()
        .class("craft-tile-body")
        .flex_col()
        .gap(3.5)
        .child(identity(card, width));
    let progress_view = card.progress();
    let state_badge = match card.working {
        Some(_) => badge(card.working_word(), "fs-t-info"),
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
    body = match progress_view {
        Some((share, line)) => body.child(running(card, share, line, inner)),
        None => body.child(actions(card, busy, inner, shared, launcher)),
    };
    div()
        .class("craft-tile")
        .w(width)
        .flex_shrink_0()
        .flex_col()
        .child(head(card, number, shared, launcher))
        .child(body)
}
