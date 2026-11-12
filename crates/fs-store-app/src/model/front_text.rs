//! Pattern: none (small pure functions that turn catalog facts into the words of the storefront; a fact the catalog does
//! not give becomes "unknown", never a guess).

use fs_store_core::{Kind, TrustLevel};

use crate::words_front as w;

/// 13631488 -> "13 MiB"; below 10 MiB one decimal ("4.2 MiB"); below 1 MiB in KiB.
#[must_use]
pub fn size_text(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * 1024;
    if bytes >= 10 * MIB {
        format!("{} MiB", bytes / MIB)
    } else if bytes >= MIB {
        format!("{}.{} MiB", bytes / MIB, (bytes % MIB) * 10 / MIB)
    } else {
        format!("{} KiB", bytes.div_ceil(KIB).max(1))
    }
}

/// The word of a kind.
#[must_use]
pub fn kind_text(kind: Option<Kind>) -> &'static str {
    match kind {
        Some(Kind::Program) => "Program",
        Some(Kind::Frontend) => "Frontend",
        Some(Kind::Component) => "Component",
        Some(Kind::Theme) => "Theme",
        None => w::UNKNOWN,
    }
}

/// The short word of a trust level, `unknown` when the catalog says nothing.
#[must_use]
pub fn trust_word(level: Option<TrustLevel>) -> &'static str {
    match level {
        Some(TrustLevel::MakerAndMirror) => w::TRUST_BOTH,
        Some(TrustLevel::OurKeyOnly) => w::TRUST_OURS,
        Some(TrustLevel::ChecksumOnly) => w::TRUST_SUM,
        None => w::UNKNOWN,
    }
}

/// What the level means, in a sentence.
#[must_use]
pub fn trust_sentence(level: Option<TrustLevel>) -> &'static str {
    match level {
        Some(TrustLevel::MakerAndMirror) => w::TRUST_BOTH_TEXT,
        Some(TrustLevel::OurKeyOnly) => w::TRUST_OURS_TEXT,
        Some(TrustLevel::ChecksumOnly) => w::TRUST_SUM_TEXT,
        None => w::TRUST_UNKNOWN_TEXT,
    }
}

/// The word or `unknown` for an empty text.
#[must_use]
pub fn or_unknown(text: &str) -> String {
    if text.trim().is_empty() {
        w::UNKNOWN.to_owned()
    } else {
        text.trim().to_owned()
    }
}

/// `text` cut at the last word that fits `max` characters, closed with an ellipsis; never in the middle of a word, and as it is when
/// it fits. A word longer than the room is cut and closed the same way.
#[must_use]
pub fn ellipsize(text: &str, max: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let keep = max.saturating_sub(1).max(1);
    let head: String = text.chars().take(keep).collect();
    // back to the end of a whole word when the cut fell inside one
    let cut = if text.chars().nth(keep).is_some_and(char::is_whitespace) {
        head
    } else {
        match head.rfind(char::is_whitespace) {
            Some(at) if at > 0 => head[..at].to_owned(),
            _ => head,
        }
    };
    format!(
        "{}\u{2026}",
        cut.trim_end_matches([' ', ',', ';', ':', '.', '-'])
    )
}

/// The first sentence of a description, at most `max` characters, closed with an ellipsis when it is longer (at a word); the one line
/// under a name.
#[must_use]
pub fn summary(text: &str, max: usize) -> String {
    let text = text.trim();
    let first = text.split_inclusive(['.', '!', '?']).next().unwrap_or(text);
    ellipsize(first, max)
}
