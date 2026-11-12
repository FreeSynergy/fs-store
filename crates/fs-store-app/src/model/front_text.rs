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

/// The first sentence of a description, at most `max` characters; the one line under a name.
#[must_use]
pub fn summary(text: &str, max: usize) -> String {
    let text = text.trim();
    let first = text.split_inclusive(['.', '!', '?']).next().unwrap_or(text);
    if first.chars().count() <= max {
        return first.trim().to_owned();
    }
    let cut: String = first.chars().take(max.saturating_sub(1)).collect();
    format!("{}...", cut.trim_end())
}
