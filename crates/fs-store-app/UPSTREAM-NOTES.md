# Was fs-components (Blinc) noch fehlt

Das Menue in der Titelleiste kommt aus `fs-components-app` (`AppWidget::set_menus`) und funktioniert. Es fehlen
zwei Dinge. Erstens gibt es keine Funktion, um das Fenster geordnet zu schliessen (zum Beispiel `AppWidget::close`
oder ein Rueckgabewert von `launch`); der Store ruft deshalb bei "Beenden" `std::process::exit(0)` auf, nachdem das
Modell geprueft hat, dass kein Auftrag laeuft. Zweitens gibt es keinen globalen Tastenkuerzel-Mechanismus
(`bind_keys` in `fs-components-widgets` ist nicht oeffentlich und gilt nur fuer das Element mit dem Fokus); Strg+Q
haengt der Store darum als `on_key_down` an die Seite und verlaesst sich darauf, dass Blinc Tasten an alle Handler
sendet. Das ist nicht in einem echten Fenster geprueft und kann fehlen, solange kein Element der Seite den Fokus
hat. Gewuenscht: ein `Shortcut`-Eintrag (Taste plus Strg) am `MenuEntry`, den die Shell selbst behandelt.
