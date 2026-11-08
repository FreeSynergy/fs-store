# Was fs-components (Blinc) noch fehlt

Das Menue in der Titelleiste kommt aus `fs-components-app` (`AppWidget::set_menus`) und funktioniert. Es fehlen
zwei Dinge. Erstens gibt es keine Funktion, um das Fenster geordnet zu schliessen (zum Beispiel `AppWidget::close`
oder ein Rueckgabewert von `launch`); der Store ruft deshalb bei "Beenden" `std::process::exit(0)` auf, nachdem das
Modell geprueft hat, dass kein Auftrag laeuft. Zweitens gibt es keinen globalen Tastenkuerzel-Mechanismus
(`bind_keys` in `fs-components-widgets` ist nicht oeffentlich und gilt nur fuer das Element mit dem Fokus); Strg+Q
haengt der Store darum als `on_key_down` an die Seite und verlaesst sich darauf, dass Blinc Tasten an alle Handler
sendet. Das ist nicht in einem echten Fenster geprueft und kann fehlen, solange kein Element der Seite den Fokus
hat. Gewuenscht: ein `Shortcut`-Eintrag (Taste plus Strg) am `MenuEntry`, den die Shell selbst behandelt.

## Zähler im Fenstertitel und in der Navigation (2026-11-08)
`Shell` hat keinen Setter für Navigationsbeschriftungen und `launch` nimmt den Fenstertitel nur einmal: "Updates (n)" und "FreeSynergy.Store (n)" zeigen den Zähler vom Start des Fensters und werden danach alt. Aktuell ist er im Banner und auf der Updates-Seite. `AppWidget::set_status` geht nur aus dem Oberflächen-Thread, deshalb zeigt die Statuszeile den Prüfstand nicht.
