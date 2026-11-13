# Changelog

## 0.0.16

### English
- **Reload is reliable.** A catalog that cannot be read (source down, no route) no longer empties the window: the old offers of that source stay, a message says what failed. A read that breaks on the worker thread (a panic in the backend) no longer leaves the button dead: it ends, tells it, and the old state stays. Tests run the real flow (ticket, read, finish) with a changing catalog, an unreachable source, a failing catalog read and a panic, each with a counter-proof.
- **Reload button:** icon only (`view-refresh`, the same size and contrast as the other buttons), tooltip "Reload catalog" (German: "Katalog neu laden").
- **One confirmation to install.** "Install" opens the question; its button is now **Install now** ("Jetzt installieren"), so it is no longer a second "Install". The question names the source, the size and the rights, and for a system install (rpm/deb) it says that the system asks for your password next. Nothing asks after "Install now".
- **Messages at the top fade and can be closed.** A hint goes after 6 s, an error after 20 s; every message has a close button; menu **Store > Messages** shows the history of the session. The clock is handed in, so tests use a fake time.
- **Start vs. Programs.** Start is an overview (updates, new, not-supported entries) with no number; the per-kind blocks that repeated the lists are gone. Each section lists ALL packages of its kind and its number is exactly what the list shows (the section title names the kind).
- **Wording:** the layouts are **frontends** (menu View > Frontend), never "face"; tiles, details and list are views.
- **Known limits:** the compact frontend has no reload button yet (it reloads on start and after source commands); a failure with the same text twice before a redraw counts as one message; the internal type and the setting key are still called `face` (renaming would break saved settings); the window drawing is tested to a picture, not by clicking in a real desktop window; the messages of the settings page (the timer note) are not in the strip.

### Deutsch
- **Neu laden ist verlässlich.** Ein Katalog, der nicht gelesen werden kann (Quelle aus, keine Route), leert das Fenster nicht mehr: Die alten Angebote dieser Quelle bleiben, eine Meldung sagt, was fehlschlug. Ein Lesevorgang, der im Arbeits-Thread abbricht (Panic im Backend), lässt den Knopf nicht mehr tot zurück: Er endet, meldet es, der alte Stand bleibt. Die Tests fahren den echten Ablauf (Ticket, Lesen, Abschluss) mit sich änderndem Katalog, unerreichbarer Quelle, fehlschlagendem Katalog und Panic, je mit Gegenprobe.
- **Neu-laden-Knopf:** nur Symbol (`view-refresh`, gleiche Größe und gleicher Kontrast wie die anderen Knöpfe), Tooltip „Katalog neu laden“ (en: „Reload catalog“).
- **Eine Bestätigung zum Installieren.** „Installieren“ öffnet die Vorschau; ihr Knopf heißt jetzt **Jetzt installieren**, ist also keine zweite „Installieren“-Frage mehr. Die Vorschau nennt Quelle, Größe und Rechte und sagt bei einer Systeminstallation (rpm/deb): „Für die Systeminstallation fragt das System gleich nach Ihrem Passwort.“ Nach „Jetzt installieren“ fragt nichts mehr.
- **Meldungen oben blenden aus und lassen sich schließen.** Ein Hinweis verschwindet nach 6 s, ein Fehler nach 20 s; jede Meldung hat einen Schließen-Knopf; Menü **Store > Meldungen** zeigt den Verlauf der Sitzung. Die Uhr wird übergeben, Tests nutzen Fake-Zeit.
- **Start gegen Programme.** Start ist der Überblick (Updates, Neu, nicht unterstützte Einträge) ohne Zahl; die Blöcke je Art, die die Listen wiederholten, entfallen. Jede Rubrik listet ALLE Pakete ihrer Art, ihre Zahl ist genau das, was die Liste zeigt (der Titel nennt die Art).
- **Wortwahl:** Die Anordnungen heißen **Frontends** (Menü Ansicht > Frontend), nie „Gesicht“; Kacheln, Details und Liste sind Ansichten.
- **Bekannte Grenzen:** Das kompakte Frontend hat noch keinen Neu-laden-Knopf (es lädt beim Start und nach Quellen-Befehlen); dieselbe Fehlermeldung zweimal vor einem Neuzeichnen zählt als eine; der interne Typ und der Einstellungsschlüssel heißen noch `face` (Umbenennen bräche gespeicherte Einstellungen); das Zeichnen des Fensters ist bis zum Bild getestet, nicht durch Klicken in einem echten Desktop-Fenster; die Meldungen der Einstellungsseite (Hinweis des Timers) stehen nicht im Streifen.

## 0.0.15

### English
- **Fix: commands follow the active install form.** A package unpacked by the store (tar.gz) and later installed as rpm kept its facet commands (`~/.local/bin/fs-tasks`, `fs-people`) on the old unpacked program (0.0.2) although the register said 0.0.7. Now facet commands point to `/usr/bin` when the package manager owns the program, and through `current/<name>` when the store unpacked it.
- **tar.gz → rpm/deb cleans up** the store's own leftovers: command links into `current/<name>`, the `current` link, old `packages/<name>/` versions and the list of files. A folder with a file the store did not put there stays and is reported; other packages and foreign commands are never touched.
- **`fs-store verify` finds "register says X, command starts Y"** (also a link to a gone file, or into the system folder while the unpacked form is active): exit code not zero, the message names the command and the repair. New **`fs-store relink [name]`** makes the commands again for the active form.
- **Fixes after review:** `fs-store verify <name>` passes on only the case "no list of files" (an rpm); a damaged list is an error (exit not zero), no longer "all 0 packages intact". `relink` does not remove the only installation: when the register's version is neither unpacked nor found in the system folder (`/usr/bin`, for tests `FS_STORE_SYSTEM`), it changes nothing and says so. A package name from `installed.toml` that is empty, `.`, `..` or holds a path separator removes nothing.
- **Known limits:** the rpm side is tested with a folder in place of `/usr/bin`, not with a real `dnf`; `relink` trusts `installed.toml`; a link that points straight into `packages/` (not through `current/`) is not checked. The form is decided only by `packages/<name>/<version>`: with tar.gz and rpm of the SAME version present, the unpacked form wins. A version folder below `packages/<name>/` that has no matching list of files is removed as a whole, without a check for foreign files (only a folder with a list is checked).

### Deutsch
- **Korrektur: Befehle folgen der aktiven Installationsform.** Ein vom Store entpacktes Paket (tar.gz), das später als rpm installiert wurde, ließ seine Facetten-Befehle (`~/.local/bin/fs-tasks`, `fs-people`) auf dem alten entpackten Programm (0.0.2), obwohl die Liste 0.0.7 sagte. Jetzt zeigen Facetten-Befehle nach `/usr/bin`, wenn die Paketverwaltung das Programm besitzt, und über `current/<name>`, wenn der Store es entpackt hat.
- **tar.gz → rpm/deb räumt auf**, nur Eigenes: Befehls-Links nach `current/<name>`, den `current`-Link, alte `packages/<name>/`-Stände und die Dateiliste. Ein Ordner mit einer Datei, die der Store nicht angelegt hat, bleibt und wird gemeldet; andere Pakete und fremde Befehle bleiben unberührt.
- **`fs-store verify` findet „Liste sagt X, Befehl startet Y“** (auch Link auf eine fehlende Datei oder in den Systemordner bei aktiver entpackter Form): Exit-Code ungleich null, die Meldung nennt Befehl und Reparatur. Neu: **`fs-store relink [name]`** legt die Befehle für die aktive Form neu an.
- **Korrekturen nach Review:** `fs-store verify <name>` lässt nur den Fall „keine Dateiliste“ (rpm) durch; eine beschädigte Liste ist ein Fehler (Exit ungleich null), nicht mehr „all 0 packages intact“. `relink` entfernt nicht die einzige Installation: Ist die Version der Liste weder entpackt noch im Systemordner (`/usr/bin`, in Tests `FS_STORE_SYSTEM`) zu finden, ändert es nichts und sagt das. Ein Paketname aus `installed.toml`, der leer, `.`, `..` ist oder einen Pfadtrenner enthält, entfernt nichts.
- **Bekannte Grenzen:** Die rpm-Seite ist mit einem Ordner statt `/usr/bin` getestet, nicht mit echtem `dnf`; `relink` vertraut `installed.toml`; ein Link, der direkt in `packages/` zeigt (nicht über `current/`), wird nicht geprüft. Die Form wird nur über `packages/<name>/<version>` entschieden: Liegen tar.gz und rpm mit GLEICHER Version vor, gewinnt die entpackte Form. Ein Versionsordner unter `packages/<name>/` ohne passende Dateiliste wird ganz entfernt, ohne Prüfung auf Fremddateien (nur ein Ordner mit Liste wird geprüft).

## 0.0.14

### English
- **Faces in the window.** Menu **View** lists the faces the code has (Storefront: Craft tiles and table; Compact: list at the left, page at the right), a dot marks the one worn. Choosing one changes
  the window at once and is remembered in `settings.toml` (`face = "storefront" | "compact"`, written atomically with the other settings). Default for a new install: the Storefront.
  `--variant b|c` stays as an override for one run (not saved). A settings file that cannot be read gives the default face and the usual warning; a face name from a newer store gives the default face and keeps the other settings.
- **Locked by `fs:min-store`, also for a program.** The reason now names the store ("needs store 9.1.0"); the window says "Needs Store 9.1.0 or newer: ..." (DE+EN) on the tile, the package page and the compact page, the
  button is off, and install is refused by the model and the engine as before (test for a program).
- `fs-components` v0.0.13 (was v0.0.11), `fs-interfaces` v0.0.19, Blinc `fs-blinc-3.1`. No `match` on `Glyph` in the store, so the five new glyphs need nothing.
- **Known limits:** the look (Craft or platform) is applied at the start, so a face chosen in the menu takes its look at the next start; there is only the Storefront and the Compact face (no third); the table of the
  Storefront shows the lock only as an off button (the reason is on the tile and the page); the menu cannot be looked at here without a window: the choice is tested through the real menu handler of the window, not by eye.

### Deutsch
- **Gesichter im Fenster.** Das Menü **Ansicht** führt die Gesichter auf, die es im Code gibt (Schaufenster: Craft-Kacheln und Tabelle; Kompakt: Liste links, Seite rechts), ein Punkt markiert das getragene. Die Wahl
  ändert das Fenster sofort und wird in `settings.toml` gemerkt (`face = "storefront" | "compact"`, atomar mit den anderen Einstellungen geschrieben). Standard bei Neuinstallation: das Schaufenster.
  `--variant b|c` bleibt als Überschreibung für einen Lauf (nicht gemerkt). Eine unlesbare Einstellungsdatei gibt das Standard-Gesicht und die übliche Warnung; ein Gesichtsname eines neueren Stores gibt das Standard-Gesicht und lässt die übrigen Einstellungen.
- **Gesperrt durch `fs:min-store`, auch bei einem Programm.** Der Grund nennt jetzt den Store („needs store 9.1.0“); das Fenster sagt „Braucht Store 9.1.0 oder neuer: ...“ (DE+EN) auf Kachel, Paketseite und kompakter Seite, der
  Knopf ist aus, und Installieren wird wie bisher von Modell und Engine verweigert (Test für ein Programm).
- `fs-components` v0.0.13 (vorher v0.0.11), `fs-interfaces` v0.0.19, Blinc `fs-blinc-3.1`. Der Store hat kein `match` auf `Glyph`, die fünf neuen Glyphen brauchen nichts.
- **Bekannte Grenzen:** Das Aussehen (Craft oder Plattform) gilt ab dem Start, ein im Menü gewähltes Gesicht bekommt sein Aussehen beim nächsten Start; es gibt nur das Schaufenster und das kompakte Gesicht (kein drittes); die Tabelle
  des Schaufensters zeigt die Sperre nur als ausgegrauten Knopf (den Grund nennen Kachel und Seite); das Menü ist hier ohne Fenster nicht anzusehen: Die Wahl ist über den echten Menü-Handler des Fensters getestet, nicht per Auge.

## 0.0.13

### English
- **Reader for catalog format 2.** An unknown or not yet installable `fs:kind` never fails the catalog or a package any more: before, `entry()` returned an error for such a
  component and it was listed only as "skipped" (the other packages loaded). Now the entry is read as far as it goes (name, summary, description, icon, version when a release
  says one) and shown **locked**: `server` and `icon-theme` are kinds of their own and say "Coming soon", any other kind is `Unsupported` ("not supported by this store version", the
  written kind stays in `CatalogEntry.kind_text`). An entry with `fs:min-store` above this store's version is locked the same way. Install is refused in three places (the button, the model
  `begin_job` and `ask_install`, and `Engine::plan`, `InstallFailure::NotInstallable`). Servers and icon sets have a section in the navigation when the catalog lists some.
- **Languages:** description, summary, picture caption and the text of a release are chosen by `xml:lang`: language of the program (`FS_LANG`, else `LANG`), then English, then the
  variant without a language, then the first (`fs_store_core::language`, `parse_catalog_in`; `parse_catalog` stays English).
- **More fields read:** `<size type="installed">`, `fs:maker`, `fs-source` of a picture, `fs:features` with `fs:feature.<key>.name|provides|requires|recommends`,
  `<provides><mediatype>`, `fs:description-wiki` with `-source-rev` and `-lang` (the block is named on the page, never loaded from the wiki). The package page shows them.
- **Wire:** `Origin.maker` is written. The contract (`fs-interfaces` v0.0.19) has no word for the kinds server/icon-theme/unknown, for features, media types, the installed size, a picture source or a
  wiki revision and language: those are not squeezed in; an entry of such a kind is returned in `skipped` with the reason, the wiki block id goes out alone. Request list for `fs-interfaces` v0.0.20:
  `PackageKind` `server @4`, `iconTheme @5`; `Feature`; `Picture.source`; `Mediatype`; installed size; `wikiBlock` language and `sourceRev`.
- **Review follow-ups of 0.0.13:** an entry with an unreadable `fs:min-store` is locked ("unreadable min-store"); a locked entry without a release file says nothing about the signature (unknown);
  locked entries are not counted as updates or new programs (`fs-store check`, `background`, "Update all" in the window and the commands); an empty `FS_LANG` no longer hides `LANG`; the feature words
  (offers, needs, can use) follow the program language (DE+EN); the wire reports a locked program as `skipped` (no lock field in the contract before v0.0.20); the engine refusal in `plan` has its own test.
- **Commands and locked packages:** `fs-store install X` / `update X` with a locked package now fail (exit not zero); only `update --all` skips it with a note.
- **Known limits:** the wiki block is not loaded; the text of a locked entry is read in the language of the program only (no re-read when it changes); an entry that is locked
  because of `fs:min-store` is compared with the version of this crate.

### Deutsch
- **Leser für Katalogformat 2.** Ein unbekannter oder noch nicht installierbarer `fs:kind` macht weder den Katalog noch ein Paket zum Fehler: Vorher gab `entry()` für so einen Eintrag einen
  Fehler zurück und er stand nur unter „übersprungen“ (die anderen Pakete wurden geladen). Jetzt wird der Eintrag so weit gelesen wie möglich (Name, Kurztext, Beschreibung, Icon, Version
  wenn ein Release eine nennt) und **gesperrt** gezeigt: `server` und `icon-theme` sind eigene Arten und sagen „Kommt bald“, jede andere Art ist `Unsupported` („von dieser Store-Version nicht
  unterstützt“, die geschriebene Art bleibt in `CatalogEntry.kind_text`). Ein Eintrag mit `fs:min-store` über der Version dieses Stores ist genauso gesperrt. Installieren wird an drei Stellen
  verweigert (Knopf, Modell `begin_job` und `ask_install`, `Engine::plan` mit `InstallFailure::NotInstallable`). Server und Icon-Sets haben einen Abschnitt in der Navigation, sobald der Katalog welche nennt.
- **Sprachen:** Beschreibung, Kurztext, Bildunterschrift und Release-Text werden nach `xml:lang` gewählt: Sprache des Programms (`FS_LANG`, sonst `LANG`), dann Englisch, dann die Fassung ohne
  Sprache, dann die erste (`fs_store_core::language`, `parse_catalog_in`; `parse_catalog` bleibt Englisch).
- **Weitere Felder gelesen:** `<size type="installed">`, `fs:maker`, `fs-source` am Bild, `fs:features` mit `fs:feature.<key>.name|provides|requires|recommends`, `<provides><mediatype>`,
  `fs:description-wiki` mit `-source-rev` und `-lang` (der Block wird auf der Seite genannt, nie aus dem Wiki geladen). Die Paketseite zeigt sie.
- **Wire:** `Origin.maker` wird geschrieben. Der Vertrag (`fs-interfaces` v0.0.19) hat kein Wort für die Arten server/icon-theme/unbekannt, für Features, Mediatypes, installierte Größe, Bildquelle und
  Wiki-Revision und -Sprache: Das wird nicht hineingepresst; ein Eintrag so einer Art kommt in `skipped` mit dem Grund zurück, die Wiki-Block-Id geht allein hinaus. Bedarfsliste für `fs-interfaces` v0.0.20:
  `PackageKind` `server @4`, `iconTheme @5`; `Feature`; `Picture.source`; `Mediatype`; installierte Größe; `wikiBlock` Sprache und `sourceRev`.
- **Nachbesserungen zu 0.0.13:** Ein Eintrag mit nicht lesbarem `fs:min-store` ist gesperrt („unreadable min-store“); ein gesperrter Eintrag ohne Release-Datei sagt nichts zur Signatur (unbekannt);
  gesperrte Einträge zählen weder als Update noch als neues Programm (`fs-store check`, `background`, „Alle aktualisieren“ im Fenster und in den Befehlen); ein leeres `FS_LANG` verdeckt `LANG` nicht mehr;
  die Feature-Wörter (bietet, braucht, nutzt) folgen der Programmsprache (DE+EN); die Wire meldet ein gesperrtes Programm als `skipped` (kein Sperrfeld im Vertrag vor v0.0.20); die Sperre in `plan` hat einen eigenen Test.
- **Befehle und gesperrte Pakete:** `fs-store install X` / `update X` mit gesperrtem Paket schlagen jetzt fehl (Exit nicht 0); nur `update --all` überspringt es mit Hinweis.
- **Bekannte Grenzen:** Der Wiki-Block wird nicht geladen; der Text eines gesperrten Eintrags wird nur in der Sprache des Programms gelesen (kein Neulesen bei Wechsel); ein Eintrag, der wegen
  `fs:min-store` gesperrt ist, wird mit der Version dieses Crates verglichen.

## 0.0.12

### English
- **Catalog facts per package** (ADR 0068): size, release date, changes per version, origin (maker, maker signature, source code), signature level, capabilities
  (offers, needs, can use; names checked against the list of `fs-interfaces`), pictures with a clear licence, the description as text or a wiki block. What the
  catalog does not say reads "unknown". The wire (`store.capnp` `Package` @7 to @18, `fs-interfaces` v0.0.19) carries the same fields; older readers see none of them.
- **Cancel and progress:** the download reports bytes and stops on "Cancel" (nothing is on the machine yet); the unpacking stops without leaving a half tree,
  the old version stays active. System packages (rpm, deb) cannot be stopped and the window says so.
- **`fs-store owner <path>` and `fs-store verify`:** the store keeps its own list of the files it unpacked (path, checksum, mode, source, signer). `verify` reports changed,
  missing, other permission bits (setuid and the like are never allowed), no longer a regular file, and unreadable files (exit code 1); strangers in the package
  folder are a warning (exit code 0). A list made later for an old install is marked as made afterwards. An existing version folder is compared with the archive on a reinstall and repaired when it
  differs.
- **Windows of the store:** variant B (Craft look: tiles, table, start page, package page, the question with the steps and the risk before any work) and variant C
  (compact: list at the left, page at the right, `--variant c`).
- **Guards:** a package that is being installed is not started a second time (window, command, timer: one lock file per package); a locked "Open" says why (German and
  English); the question before an install names a need that nobody offers; empty blocks are one line.
- **Known limits:** no Breeze look yet (variant C wears the platform theme); no pause of a running job; downloads from `file://` report once, not in pieces; a failed restart of
  an old background server after a failed update is not reported (`batch.rs`); the icon is stored only by the commands, not by the window; removals and rpm/deb installs take no
  per-package lock (the package manager has its own); the maker's signature is as the signed catalog says, the store does not check it itself.

### Deutsch
- **Katalogangaben je Paket** (ADR 0068): Größe, Datum, Änderungen je Version, Herkunft (Hersteller, Herstellersignatur, Quellcode), Signaturstufe, Fähigkeiten (bietet, braucht,
  nutzt; Namen gegen die Liste von `fs-interfaces` geprüft), Bilder mit klarer Lizenz, Beschreibung als Text oder Wiki-Block. Was der Katalog nicht sagt, steht als „unknown“ da. Die Wire
  (`store.capnp` `Package` @7 bis @18, `fs-interfaces` v0.0.19) trägt dieselben Felder; ältere Leser sehen sie nicht.
- **Abbrechen und Fortschritt:** Der Download meldet Bytes und stoppt auf „Abbrechen“ (noch nichts auf der Maschine); das Entpacken stoppt ohne halben Baum, die alte Version bleibt aktiv.
  Systempakete (rpm, deb) lassen sich nicht stoppen, das Fenster sagt es.
- **`fs-store owner <Pfad>` und `fs-store verify`:** Der Store führt eine eigene Liste der entpackten Dateien (Pfad, Prüfsumme, Modus, Quelle, Signierer). `verify` meldet geänderte, fehlende,
  Dateien mit anderen Rechten (setuid u. ä. nie erlaubt), Dateien die keine normale Datei mehr sind und nicht lesbare (Exit 1); Fremdes im Paketordner ist eine Warnung (Exit 0). Eine
  nachträglich erzeugte Liste ist als nachträglich markiert. Ein vorhandener Versionsordner wird bei Neuinstallation mit dem Archiv verglichen und bei Abweichung repariert.
- **Fenster:** Variante B (Craft: Kacheln, Tabelle, Startseite, Paketseite, die Frage mit Schritten und Risiko vor jeder Arbeit) und Variante C (kompakt: Liste links, Seite rechts, `--variant c`).
- **Schutz:** Ein Paket in Installation wird nicht ein zweites Mal gestartet (Fenster, Befehl, Zeitgeber: eine Sperrdatei je Paket); ein gesperrtes „Öffnen“ sagt warum (Deutsch und Englisch);
  die Frage vor dem Installieren nennt einen Bedarf, den niemand anbietet; leere Blöcke sind eine Zeile.
- **Bekannte Grenzen:** noch kein Breeze-Aussehen (Variante C im Plattform-Stil); keine Pause eines laufenden Auftrags; `file://`-Downloads melden einmal, nicht in Stücken; ein fehlgeschlagener
  Neustart eines alten Hintergrunddienstes nach misslungenem Update wird nicht gemeldet (`batch.rs`); das Symbol legt nur der Befehl ab, nicht das Fenster; Entfernen und rpm/deb-Installationen
  haben keine Sperre je Paket (der Paketmanager hat eine eigene); die Herstellersignatur gilt nach Angabe des signierten Katalogs, der Store prüft sie nicht selbst.
