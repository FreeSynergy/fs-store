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

## Eine Seite mit Karten (2026-11-08)
Was fehlt oder nicht geprueft ist. Erstens Hover: `blinc_css` kennt keinen `:hover`-Selektor, den wir gefunden haben; die Hervorhebung der Karte unter dem Mauszeiger laeuft deshalb ueber `on_hover_enter`/`on_hover_leave` am Div, aendert den Zustand im Modell und loest ein neues Zeichnen aus (nur wenn sich wirklich etwas aendert). Im Bild von `--snapshot` ist kein Mauszeiger, die Hervorhebung ist nur im Modelltest geprueft, nicht in einem echten Fenster. Zweitens das Drei-Punkte-Menue: `blinc_cn::dropdown_menu_custom` wird benutzt; der Ausloeser ist das Zeichen U+22EE (drei Punkte uebereinander) in einem kleinen Quadrat, nicht drei gezeichnete Punkte (drei kleine Divs wurden schief dargestellt). Das aufgeklappte Menue erscheint nicht im Bild von `--snapshot` (Overlay) und ist in keinem echten Fenster geprueft. Drittens das Logo: `blinc_layout::img` gibt es, aber der Store laedt und speichert das Icon des Katalogeintrags (mit SHA-256) noch nicht; bis dahin zeichnet er einen Platzhalter (erster Buchstabe auf einem Farbblock, Farbe aus dem Namen). Das ist eine Aufgabe des Stores, keine von Blinc. Viertens: `ButtonBuilder` hat keine Breite; die Knoepfe der Karte bekommen die Klasse `fs-wide` (`width: 100%`) in einem Div mit fester Breite. Fuenftens: Es gibt nur noch einen `Screen`, damit die Shell keine Navigation zeigt; Quellen, Einstellungen und Info wechseln ueber das Modell (`View`) und das Menue, "Zurueck" ist ein Knopf auf der Seite. Sechstens klickbar fuer die Beschreibung ist nur der obere Teil der Karte (Logo und Name), nicht die ganze Karte, weil nicht sicher ist, ob ein Klick auf einen Knopf in der Karte auch beim Karten-Div ankommt.
