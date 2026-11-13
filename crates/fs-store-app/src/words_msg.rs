//! Pattern: none (the words of the message strip, the message history, the reload button and the question before an install, in
//! English and German).

use crate::words_lang::Lang;

/// Tooltip of the reload button (it has an icon and no label).
#[must_use]
pub fn reload_tip(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Reload catalog",
        Lang::De => "Katalog neu laden",
    }
}

/// Tooltip of the reload button while the catalogs are being read.
#[must_use]
pub fn reload_busy(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Reading the catalog...",
        Lang::De => "Der Katalog wird gelesen ...",
    }
}

/// Tooltip of the close button of a message.
#[must_use]
pub fn close_tip(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Close message",
        Lang::De => "Meldung schlie\u{df}en",
    }
}

/// The close button of a message: a cross.
pub const CLOSE_MARK: &str = "\u{d7}";

/// The menu entry (and the page heading) of the message history.
#[must_use]
pub fn history_menu(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Messages",
        Lang::De => "Meldungen",
    }
}

/// The history when there is no message.
#[must_use]
pub fn history_empty(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "No messages yet.",
        Lang::De => "Noch keine Meldungen.",
    }
}

/// Before an install: the password of the system comes next.
#[must_use]
pub fn password_notice(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "For the system install the system will ask for your password in a moment.",
        Lang::De => "F\u{fc}r die Systeminstallation fragt das System gleich nach Ihrem Passwort.",
    }
}

/// The confirming button of the question before an install.
#[must_use]
pub fn install_now(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Install now",
        Lang::De => "Jetzt installieren",
    }
}

/// The confirming button of the question before an update.
#[must_use]
pub fn update_now(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Update now",
        Lang::De => "Jetzt aktualisieren",
    }
}

/// Labels of the facts line of the question: the source, the size, the rights. Placeholders: `{source}`, `{size}`, `{rights}`.
#[must_use]
pub fn facts_line(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Source: {source} - Size: {size} - Rights: {rights}",
        Lang::De => "Quelle: {source} - Gr\u{f6}\u{df}e: {size} - Rechte: {rights}",
    }
}

/// The rights of an install that stays in the person's own folder.
#[must_use]
pub fn rights_own(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "only your own folders",
        Lang::De => "nur Ihre eigenen Ordner",
    }
}

/// The rights of an install for the whole system.
#[must_use]
pub fn rights_system(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "whole system (administrator password)",
        Lang::De => "ganzes System (Administrator-Passwort)",
    }
}

/// The start page when nothing waits (no update, nothing new).
#[must_use]
pub fn start_quiet(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Everything is up to date, and nothing is new. All programs are under Programs.",
        Lang::De => "Alles ist aktuell, nichts ist neu. Alle Programme stehen unter Programme.",
    }
}
