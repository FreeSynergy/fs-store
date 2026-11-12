# Changelog

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
