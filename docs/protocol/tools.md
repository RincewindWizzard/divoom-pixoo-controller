# Tools, Alarm und Sonstiges

## Tools: `0x72` SET_TOOL

Das erste Argument wählt das Tool:

| Tool | Befehl | Client |
| --- | --- | --- |
| `00` Timer | `72 00 S` mit `S` = `00` stopp, `01` start (die Community sendet `02` als Voreinstellung; Bedeutung ungeklärt) | ✅ `timer` |
| `01` Scoreboard (Ditoo) | `72 01 01 RR RR BB BB` | 📖 |
| `02` Lärmmesser | `72 02 S` mit `S` = `01` start, `02` stopp | ✅ (Bibliothek) |
| `03` Countdown | `72 03 S MM SS` mit `S` = `00`/`01` stopp/start, Minuten, Sekunden | ✅ `countdown` |

Beim Pixoo selbst läuft das Scoreboard über den Kanal 6 (`45 06 …`, siehe
[Kanäle](channels.md)).

## Wecker: `0x43` SET_ALARM 📖

```text
43 | SLOT | EN | HH | MM | DAYS | MODE | TRIG | F1 F2 | VOL
```

| Feld | Bedeutung |
| --- | --- |
| `SLOT` | Weckerplatz (0-basiert, Anzahl je Gerät) |
| `EN` | `01` aktiv, `00` Slot löschen |
| `HH MM` | Uhrzeit |
| `DAYS` | Bitmaske der Wochentage: So `01`, Mo `02`, Di `04`, Mi `08`, Do `10`, Fr `20`, Sa `40` |
| `MODE` | Wecker-Modus (App-Liste; Pixoo: immer `00`) |
| `TRIG` | Auslöse-Modus (App-Liste; Pixoo: immer `00`) |
| `F1 F2` | FM-Frequenz (Kodierung wie bei `SET_RADIO_FREQUENCY`), sonst `00 00` |
| `VOL` | Lautstärke (Pixoo-Client sendet `01`) |

## Erinnerung: `0x54` SET_MEMORIAL 📖

```text
54 | SLOT | EN | MON | DAY | HH | MM | ANIM | TEXT (16 × 2 Byte)
```

* `EN` `01` setzt, `00` löscht den Slot (Datum dann `00 00 00 00`)
* `ANIM` `01` = animierter Hinweis
* Text: bis zu 15 Zeichen, **zwei Bytes je Zeichen** (`00 CC`), mit `00 00`
  auf 16 Zeichen aufgefüllt. Nur ASCII ist belegt.

## Einschlaf-Modus: `0x40` SET_SLEEPTIME 📖

```text
40 | MIN | MODE | EN | F1 F2 | VOL | R G B | BRIGHT
```

`MIN` Laufzeit in Minuten (Voreinstellung der Community 120), `MODE` Klang,
`EN` start/stopp, `F1 F2` FM-Frequenz, `VOL` Lautstärke 0–100, `R G B` und
`BRIGHT` Farbe und Helligkeit des Displays. Auf dem Pixoo ist die Funktion
begrenzt (kein Lautsprecher).

## Zeit und Wetter

Siehe [Befehlsreferenz](commands.md): `SET_DATE_TIME` (`0x18`),
`SET_TIME_TYPE` (`0x2D`), `SET_TEMP` (`0x5F`), `SET_TEMP_TYPE` (`0x2B`).
Die Reihenfolge der Community beim Wetter: erst `SET_TEMP`, dann
(wenn die Einheit bekannt ist) `SET_TEMP_TYPE`. Fahrenheit wird vor dem
Senden in °C umgerechnet.

## Was die App noch kann (undokumentiert)

Die Divoom-App kennt viele weitere Funktionen (Pixel-Editor mit Live-Vorschau,
Galerie-Download, Musik-Visualisierung, Lichtpaletten usw.). Zu den meisten ist
das Bluetooth-Protokoll nicht öffentlich beschrieben. Eine Möglichkeit, es
herauszufinden: Android-**HCI-Snoop-Log** aktivieren, in der App eine Aktion
auslösen und das Log in Wireshark ansehen (Filter `btspp` bzw. `btrfcomm`).
Gefundene Befehle bitte mit `pixoo raw` testen und hier ergänzen.
