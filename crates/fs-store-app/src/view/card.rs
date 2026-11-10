//! Pattern: Adapter (one [`Card`] of the model as Blinc elements): the logo block and the name on top, the
//! buttons below, the three-dot menu beside them. Every click calls a model command.

use std::sync::Arc;

use blinc_cn::components::button::{ButtonBuilder, ButtonVariant};
use blinc_core::{Brush, Color, Gradient, GradientSpace, GradientSpread, GradientStop, Point};
use blinc_layout::div::{Div, div};
use blinc_layout::image::img;
use blinc_layout::prelude::text;

use crate::launcher::Launcher;
use crate::model::{Card, OpenWidth, Working};
use crate::words;

use super::job::{start_job, start_uninstall};
use super::more::more_menu;
use super::shell::{Shared, button, locked, rebuild};

/// Width of a card, in pixels.
pub(super) const CARD_W: f32 = 240.0;
/// Width of the button row inside a card: the card without its padding.
const ROW_W: f32 = CARD_W - 24.0;
/// The three-dot menu, with the gap before it.
const MORE_W: f32 = 32.0 + 6.0;
/// The buttons share what is left.
const BUTTONS_W: f32 = ROW_W - MORE_W;
const HALF_W: f32 = (BUTTONS_W - 6.0) / 2.0;

fn colour(rgb: (u8, u8, u8)) -> Color {
    Color::rgb(
        f32::from(rgb.0) / 255.0,
        f32::from(rgb.1) / 255.0,
        f32::from(rgb.2) / 255.0,
    )
}

/// Side of the logo square, in pixels.
const LOGO: f32 = 56.0;
/// Corner radius of the logo square.
const LOGO_RADIUS: f32 = 14.0;

/// The soft gradient of the stand-in, from the top left to the bottom right.
fn gradient(card: &Card) -> Gradient {
    Gradient::Linear {
        start: Point::new(0.0, 0.0),
        end: Point::new(1.0, 1.0),
        stops: vec![
            GradientStop::new(0.0, colour(card.placeholder.rgb)),
            GradientStop::new(1.0, colour(card.placeholder.rgb_end)),
        ],
        space: GradientSpace::ObjectBoundingBox,
        spread: GradientSpread::Pad,
    }
}

/// The logo: the stored icon (a PNG) when there is one, else a rounded square with a gradient from the name's
/// colour and the first letter in the middle. Both have the same size.
fn logo(card: &Card) -> Div {
    let frame = div()
        .w(LOGO)
        .h(LOGO)
        .flex_shrink_0()
        .rounded(LOGO_RADIUS)
        .overflow_clip();
    if let Some(path) = &card.icon {
        return frame.child(
            img(path.to_string_lossy().into_owned())
                .size(LOGO, LOGO)
                .cover(),
        );
    }
    frame
        .background(Brush::Gradient(gradient(card)))
        .items_center()
        .justify_center()
        .child(
            text(card.placeholder.letter.to_string())
                .size(28.0)
                .bold()
                .color(Color::WHITE),
        )
}

/// A button of the row with a fixed width.
fn slot(width: f32, button: ButtonBuilder) -> Div {
    div()
        .w(width)
        .flex_shrink_0()
        .flex_col()
        .child(button.build_component().class("fs-wide"))
}

fn head(card: &Card, shared: &Shared) -> Div {
    let mut names = div()
        .flex_col()
        .flex_grow()
        .min_w(0.0)
        .gap(0.5)
        .child(text(card.title.clone()).bold().no_wrap())
        .child(text(card.version_line()).class("fs-store-sub").no_wrap());
    if card.is_new {
        names = names.child(text(words::MARK_NEW).class("fs-card-badge"));
    }
    let (shared, name) = (Arc::clone(shared), card.name.clone());
    div()
        .class("fs-card-head")
        .cursor_pointer()
        .child(logo(card))
        .child(names)
        .on_click(move |_| {
            locked(&shared).select(&name);
            rebuild();
        })
}

fn actions(card: &Card, busy: bool, shared: &Shared, launcher: &Arc<dyn Launcher>) -> Div {
    let mut row = div().flex_row().items_center().gap(1.5);
    let name = card.name.clone();
    if card.buttons.install {
        let shared = Arc::clone(shared);
        let name = name.clone();
        row = row.child(slot(
            BUTTONS_W,
            button(
                &format!("install-{name}"),
                words::BUTTON_INSTALL,
                ButtonVariant::Primary,
                !busy,
            )
            .on_click(move |_| start_job(&shared, &name)),
        ));
    }
    if let Some(width) = card.buttons.open {
        let width = match width {
            OpenWidth::Full => BUTTONS_W,
            OpenWidth::Half => HALF_W,
        };
        let (shared, launcher, name) = (Arc::clone(shared), Arc::clone(launcher), name.clone());
        row = row.child(slot(
            width,
            button(
                &format!("open-{name}"),
                words::BUTTON_OPEN,
                ButtonVariant::Secondary,
                true,
            )
            .on_click(move |_| {
                locked(&shared).open(&name, None, launcher.as_ref());
                rebuild();
            }),
        ));
    }
    if card.buttons.update {
        let (shared, name) = (Arc::clone(shared), name.clone());
        row = row.child(slot(
            HALF_W,
            button(
                &format!("update-{name}"),
                words::BUTTON_UPDATE,
                ButtonVariant::Primary,
                !busy,
            )
            .on_click(move |_| start_job(&shared, &name)),
        ));
    }
    row.child(more_menu(card, shared, launcher))
}

/// The question before an uninstall: yes or no.
fn confirm(card: &Card, shared: &Shared) -> Div {
    let (yes, no) = (Arc::clone(shared), Arc::clone(shared));
    div()
        .flex_col()
        .gap(1.5)
        .child(text(words::fill(
            words::UNINSTALL_ASK,
            &[("name", &card.title)],
        )))
        .child(
            div()
                .flex_row()
                .gap(1.5)
                .child(
                    button(
                        &format!("yes-{}", card.name),
                        words::BUTTON_UNINSTALL_YES,
                        ButtonVariant::Destructive,
                        true,
                    )
                    .on_click(move |_| start_uninstall(&yes)),
                )
                .child(
                    button(
                        &format!("no-{}", card.name),
                        words::BUTTON_CANCEL,
                        ButtonVariant::Outline,
                        true,
                    )
                    .on_click(move |_| {
                        locked(&no).cancel_uninstall();
                        rebuild();
                    }),
                ),
        )
}

/// What a busy card says instead of its buttons.
fn working(what: Working) -> Div {
    let line = match what {
        Working::Removing | Working::Asking => words::REMOVING.to_owned(),
        Working::Job(_, phase) => words::phase_name(phase).to_owned(),
    };
    div().class("fs-card-working").child(text(line))
}

/// The card of one program.
pub(super) fn card(card: &Card, busy: bool, shared: &Shared, launcher: &Arc<dyn Launcher>) -> Div {
    let state_class = if card.selected {
        "fs-card-selected"
    } else {
        "fs-card-rest"
    };
    let foot = match card.working {
        Some(Working::Asking) => confirm(card, shared),
        Some(what) => working(what),
        None => actions(card, busy, shared, launcher),
    };
    let mut body = div()
        .class("fs-card")
        .class(state_class)
        .w(CARD_W)
        .flex_col()
        .gap(3.0)
        .child(head(card, shared));
    if let Some(why) = &card.failure {
        body = body.child(text(why.clone()).class("fs-store-sub"));
    }
    body.child(foot)
}
