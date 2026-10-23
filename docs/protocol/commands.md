# Befehlsreferenz

Alle Befehle werden wie unter [Überblick](overview.md) gerahmt. Die Spalte
*Client* sagt, ob `pixoo` den Befehl kennt:
✅ = als Unterbefehl bzw. Methode umgesetzt, 📖 = nur dokumentiert
(über `pixoo raw` ausprobierbar).

| Byte | Name | Zweck | Client |
| --- | --- | --- | --- |
| `0x01` | SET_JSON | JSON-Kommandos (WLAN-Geräte, Details unbekannt) | 📖 |
| `0x05` | SET_RADIO | Audioquelle/Radio ein/aus | 📖 |
| `0x08` | SET_VOLUME | Lautstärke | 📖 |
| `0x0A` | SET_PLAYSTATE | Play/Pause | 📖 |
| `0x12` | SET_PLAYTRACK | Vorheriger/nächster Titel | 📖 |
| `0x17` | SET_GAME_KEYDOWN | Spiel: Taste gedrückt | 📖 |
| `0x18` | SET_DATE_TIME | Datum und Uhrzeit setzen | ✅ `set-time` |
| `0x21` | SET_GAME_KEYUP | Spiel: Taste losgelassen | 📖 |
| `0x23` | SET_KEYBOARD | Tastatur-LEDs (Ditoo) | 📖 |
| `0x26` | SET_HOT | „Hot“-Diashow auf der Uhr | 📖 |
| `0x2B` | SET_TEMP_TYPE | °C / °F | 📖 |
| `0x2D` | SET_TIME_TYPE | 12 h / 24 h | ✅ (bei `clock`) |
| `0x32` | SET_LIGHTNESS | Helligkeit (alternativer Befehl, ältere Geräte) | 📖 |
| `0x40` | SET_SLEEPTIME | Einschlaf-Modus | 📖 |
| `0x43` | SET_ALARM | Wecker | 📖 |
| `0x44` | SET_IMAGE | Einzelbild anzeigen | ✅ `pixel`, `fill`, `image` |
| `0x45` | SET_VIEW | Kanal wechseln und konfigurieren | ✅ `clock`, `light`, `channel`, … |
| `0x46` | GET_VIEW | Aktuellen Kanal abfragen | ✅ `get-view` |
| `0x47` | SET_LIGHT_COLOR | Lichtfarbe (alt) | 📖 |
| `0x49` | SET_ANIMATION_FRAME | Animation in Stücken übertragen | ✅ `image *.gif` |
| `0x4C` | SET_TEMP_UNIT | Temperatureinheit (alt) | 📖 |
| `0x54` | SET_MEMORIAL | Erinnerungs-Slot | 📖 |
| `0x5F` | SET_TEMP | Temperatur und Wetter | ✅ `weather` |
| `0x61` | SET_RADIO_FREQUENCY | UKW-Frequenz | 📖 |
| `0x72` | SET_TOOL | Timer, Countdown, Lärmmesser | ✅ |
| `0x74` | SET_BRIGHTNESS | Helligkeit 0–100 | ✅ `brightness` |
| `0x88` | SET_GAME_KEYPRESS | Spiel: Start | 📖 |
| `0x8B` | SET_GIF | Animation (Variante für größere Geräte) | 📖 |
| `0xA0` | SET_GAME | Spiel starten/beenden | 📖 |
| `0xBD` | SET_DESIGN | Design-Tab im Design-Kanal | ✅ `design` |

Die Namen sind die der Community-Implementierung
(hass-divoom); Divoom selbst nennt sie in der App-Software anders. Die Liste
ist nicht vollständig – die Divoom-App kennt noch viele weitere Befehle
(Cloud-Galerie, Pixel-Editor, Mikro-Funktionen usw.).

## Kurzreferenz der einfachen Befehle

### `0x74` SET_BRIGHTNESS ✅

| Byte | Bedeutung |
| --- | --- |
| 0 | Helligkeit 0–100 (`0x00`–`0x64`) |

### `0x18` SET_DATE_TIME ✅

| Byte | Bedeutung |
| --- | --- |
| 0 | Jahr mod 100 (2026 → `26`) |
| 1 | Jahr / 100 (2026 → `20`) |
| 2 | Monat 1–12 |
| 3 | Tag 1–31 |
| 4 | Stunde 0–23 |
| 5 | Minute |
| 6 | Sekunde |
| 7 | Wochentag, 0 = Sonntag … 6 = Samstag |

Beispiel (2026-10-23 19:45:00, Freitag):
`01 0B 00 18 1A 14 0A 17 13 2D 00 05 B7 00 02`

### `0x5F` SET_TEMP ✅

| Byte | Bedeutung |
| --- | --- |
| 0 | Temperatur in °C als vorzeichenbehaftetes Byte (−5 → `0xFB`) |
| 1 | Wetter: `01` klar, `03` bewölkt, `05` Gewitter, `06` Regen, `08` Schnee, `09` Nebel |

Wird auf dem Uhr-Kanal angezeigt, wenn „Wetter“/„Temperatur“ aktiviert ist.

### `0x2D` SET_TIME_TYPE ✅

| Byte | Bedeutung |
| --- | --- |
| 0 | `00` = 12 h, `01` = 24 h |

### `0x2B` SET_TEMP_TYPE 📖

| Byte | Bedeutung |
| --- | --- |
| 0 | `00` = °C, `01` = °F |

### `0x26` SET_HOT 📖

| Byte | Bedeutung |
| --- | --- |
| 0 | `00`/`01`: Diashow der „besten Bilder“ auf der Uhr aus/an |

## Audio-Befehle (nur Geräte mit Lautsprecher) 📖

| Befehl | Argumente |
| --- | --- |
| `0x08` SET_VOLUME | 1 Byte: Lautstärke 0–15 (Prozent × 15 / 100) |
| `0x0A` SET_PLAYSTATE | 1 Byte: `00` Pause, `01` Play |
| `0x12` SET_PLAYTRACK | 1 Byte: `00` vorheriger, `01` nächster Titel |
| `0x05` SET_RADIO | 1 Byte Quelle: `00` Bluetooth (bzw. Radio aus), `01` FM, `02` Line-in, `03` SD-Karte, `07` USB |
| `0x61` SET_RADIO_FREQUENCY | 2 Byte, siehe unten |

FM-Frequenz in MHz: `f10 = f * 10`; ist `f10 > 1000`: Bytes `[f10 − 1000, f10 / 100]`,
sonst `[f10 mod 100, f10 / 100]` (alles ganzzahlig).

## Spiele 📖

| Befehl | Argumente |
| --- | --- |
| `0xA0` SET_GAME | 2 Byte: `00 00` beenden; `01 NN` Spiel Nummer NN starten |
| `0x88` SET_GAME_KEYPRESS | keine – „Go“/Start |
| `0x17` SET_GAME_KEYDOWN | 1 Byte Taste: `01` links, `02` rechts, `03` hoch, `04` runter, `05` OK |
| `0x21` SET_GAME_KEYUP | 1 Byte Taste wie oben |

Eine Taste wird als KEYDOWN, ca. 100 ms warten, KEYUP gesendet.

## Ditoo-Tastatur 📖

`0x23` SET_KEYBOARD, 1 Byte: `00` vorheriger Effekt, `01` nächster Effekt,
`02` ein/aus. Nur beim Ditoo vorhanden.
