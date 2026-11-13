//! Pattern: none (the few words that have an English and a German version; the rest of the window is English until the whole dictionary
//! exists). `words_front` re-exports them.

/// The language of the words that have a German version: `de` when `FS_LANG` (else `LANG`) starts with `de`, else English. Only the
/// words below have both; the rest of the window is English until the whole dictionary exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    /// English.
    En,
    /// German.
    De,
}

impl Lang {
    /// The language of the person's environment.
    #[must_use]
    pub fn current() -> Self {
        let v = fs_store_core::language::from_environment();
        if v.to_ascii_lowercase().starts_with("de") {
            Lang::De
        } else {
            Lang::En
        }
    }
}

/// Why "Open" is locked for a package that has no window of its own (a component, a theme).
#[must_use]
pub fn no_window(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "A component has no window of its own, so there is nothing to open.",
        Lang::De => "Eine Komponente hat kein eigenes Fenster, es gibt nichts zu \u{f6}ffnen.",
    }
}

/// The three words before the capabilities of a feature: what it offers, needs and can use.
#[must_use]
pub fn feature_words(lang: Lang) -> [&'static str; 3] {
    match lang {
        Lang::En => ["offers", "needs", "can use"],
        Lang::De => ["bietet", "braucht", "nutzt"],
    }
}

/// Why "Install" is locked for a kind the catalog lists but this store version does not install yet (a server, an icon set).
#[must_use]
pub fn coming_soon(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Coming soon: this store version cannot install this kind of package yet.",
        Lang::De => {
            "Kommt bald: Diese Version des Stores kann diese Art Paket noch nicht installieren."
        }
    }
}

/// Why "Install" is locked for a kind this store version does not know at all.
#[must_use]
pub fn not_supported(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Not supported by this store version: update the store to install it.",
        Lang::De => {
            "Von dieser Store-Version nicht unterst\u{fc}tzt: Aktualisiere den Store, um es zu installieren."
        }
    }
}

/// Title of the menu that holds the faces.
#[must_use]
pub fn menu_view(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "View",
        Lang::De => "Ansicht",
    }
}

/// The menu line of a face: its name and a short description (`{mark}` is filled by the menu: a dot for the face in use).
#[must_use]
pub fn face_line(lang: Lang, face: fs_store_sources::Face) -> &'static str {
    use fs_store_sources::Face;
    match (lang, face) {
        (Lang::En, Face::Storefront) => {
            "{mark} Frontend: Storefront (tiles or a table, Craft look)"
        }
        (Lang::En, Face::Compact) => {
            "{mark} Frontend: Compact (list at the left, page at the right)"
        }
        (Lang::De, Face::Storefront) => {
            "{mark} Frontend: Schaufenster (Kacheln oder Tabelle, Craft-Look)"
        }
        (Lang::De, Face::Compact) => "{mark} Frontend: Kompakt (Liste links, Seite rechts)",
    }
}

/// Why "Install" is locked for an entry that wants a newer store. Placeholder: `{version}`.
#[must_use]
pub fn needs_store(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Needs Store {version} or newer: update the store to install it.",
        Lang::De => {
            "Braucht Store {version} oder neuer: Aktualisiere den Store, um es zu installieren."
        }
    }
}

/// The warning line of the question before an install when a need has no provider. Placeholders: `{need}`, `{title}`.
#[must_use]
pub fn missing_need(lang: Lang) -> &'static str {
    match lang {
        Lang::En => {
            "Missing: {need} is offered by no package. {title} may not work until one is installed."
        }
        Lang::De => {
            "Fehlt: {need} bietet kein Paket an. {title} funktioniert vielleicht nicht, bis eines installiert ist."
        }
    }
}

/// The note when a need can be met by several packages. Placeholders: `{need}`, `{by}`.
#[must_use]
pub fn choice_need(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Choice: {need} is offered by {by}; you choose which one to install.",
        Lang::De => "Auswahl: {need} bieten {by}; Sie w\u{e4}hlen, welches Sie installieren.",
    }
}

/// Which blocks of a package page have nothing to show (four independent facts).
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EmptyBlocks {
    /// Nothing offered.
    pub offers: bool,
    /// Nothing needed.
    pub needs: bool,
    /// Nothing it can use.
    pub uses: bool,
    /// No pictures.
    pub pictures: bool,
}

/// The one line for the blocks that have nothing to show. The parts are only those that are empty.
#[must_use]
pub fn nothing_line(lang: Lang, empty: EmptyBlocks) -> Option<String> {
    let EmptyBlocks {
        offers,
        needs,
        uses,
        pictures,
    } = empty;
    let mut parts: Vec<&str> = Vec::new();
    match lang {
        Lang::En => {
            if offers {
                parts.push("Offers nothing");
            }
            if needs {
                parts.push("Needs nothing");
            }
            if uses {
                parts.push("Uses nothing extra");
            }
            if pictures {
                parts.push("No pictures");
            }
        }
        Lang::De => {
            if offers {
                parts.push("Bietet nichts an");
            }
            if needs {
                parts.push("Braucht nichts");
            }
            if uses {
                parts.push("Nutzt nichts zus\u{e4}tzlich");
            }
            if pictures {
                parts.push("Keine Bilder");
            }
        }
    }
    (!parts.is_empty()).then(|| parts.join(" \u{b7} "))
}
