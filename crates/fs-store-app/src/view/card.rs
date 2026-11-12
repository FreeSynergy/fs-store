//! Pattern: Adapter (the logo of a package: the stored icon, or a rounded square of its accent colour with the first letter; both have the
//! size asked for). The tiles and the table rows are in `front/`.

use fs_components::blinc::core::{Brush, Color};
use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::image::img;
use fs_components::blinc::layout::prelude::text;

use crate::model::Card;

/// Side of the logo square, in pixels.
const LOGO: f32 = 56.0;
/// Corner radius of the logo square.
const LOGO_RADIUS: f32 = 14.0;

/// The logo at table size.
pub(super) fn logo_small(card: &Card) -> Div {
    sized_logo(card, 36.0, 9.0, 18.0)
}

/// The logo: the stored icon (a PNG) when there is one, else a rounded square in the accent colour of the name with
/// its first letter in black. Both have the same size.
pub(super) fn logo(card: &Card) -> Div {
    sized_logo(card, LOGO, LOGO_RADIUS, 28.0)
}

/// The logo `side` pixels square with the given corner and letter size.
pub(super) fn sized_logo(card: &Card, side: f32, radius: f32, letter: f32) -> Div {
    let frame = div()
        .w(side)
        .h(side)
        .flex_shrink_0()
        .rounded(radius)
        .overflow_clip();
    if let Some(path) = &card.icon {
        return frame.child(
            img(path.to_string_lossy().into_owned())
                .size(side, side)
                .cover(),
        );
    }
    let accent = fs_components_styles::styles::craft::ACCENTS
        [fs_components_styles::styles::craft::accent_index(&card.name)];
    frame
        .background(Brush::Solid(Color::rgb(
            f32::from(accent.r) / 255.0,
            f32::from(accent.g) / 255.0,
            f32::from(accent.b) / 255.0,
        )))
        .items_center()
        .justify_center()
        .child(
            text(card.placeholder.letter.to_string())
                .size(letter)
                .bold()
                .color(Color::BLACK),
        )
}

/// The logo of variant C: the stored icon, or a rounded square in the colour of the name with its first letter in white.
pub(super) fn logo_plain(card: &Card, side: f32) -> Div {
    let frame = div()
        .w(side)
        .h(side)
        .flex_shrink_0()
        .rounded(side / 6.0)
        .overflow_clip();
    if let Some(path) = &card.icon {
        return frame.child(
            img(path.to_string_lossy().into_owned())
                .size(side, side)
                .cover(),
        );
    }
    let (r, g, b) = card.placeholder.rgb;
    frame
        .background(Brush::Solid(Color::rgb(
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
        )))
        .items_center()
        .justify_center()
        .child(
            text(card.placeholder.letter.to_string())
                .size(side * 0.5)
                .bold()
                .color(Color::WHITE),
        )
}
