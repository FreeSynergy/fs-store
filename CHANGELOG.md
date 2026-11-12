# Changelog

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
