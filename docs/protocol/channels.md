# Kanäle (Views)

Die Anzeige des Pixoo kennt „Kanäle“ (in der App die Tabs unten). Befehl
`0x45` SET_VIEW wechselt den Kanal; das erste Argument ist die Kanalnummer.
Weitere Argumente konfigurieren den Kanal. Ohne sie wird nur umgeschaltet.

| Nr. | Kanal | Client |
| --- | --- | --- |
| `00` | Uhr (Clock) | ✅ |
| `01` | Licht (Light) | ✅ |
| `02` | Cloud (Divoom-Galerie) | ✅ umschalten |
| `03` | Effekte (VJ Effects) | ✅ |
| `04` | Visualisierung | ✅ |
| `05` | Design (eigene Bilder/Animationen) | ✅ |
| `06` | Scoreboard | ✅ |

Nicht jedes Gerät hat alle Kanäle oder die gleiche Anzahl an Effekten; die
Nummern richten sich nach der Reihenfolge in der App.

## Uhr (`45 00 …`) ✅

```text
45 00 H24 STYLE 01 WEATHER TEMP CAL [R G B]
```

| Byte | Bedeutung |
| --- | --- |
| `00` | Kanal Uhr |
| `H24` | `00` 12 h, `01` 24 h |
| `STYLE` | Zifferblatt: `00` Vollbild, `01` Regenbogen, `02` mit Rahmen, `03` analog eckig, `04` Vollbild negativ, `05` analog rund, `06` breit (Pixoo; weitere je Modell, bis 15) |
| `01` | „Uhr aktiv“ (feste 1) |
| `WEATHER` | Wetter anzeigen (0/1) |
| `TEMP` | Temperatur anzeigen (0/1) |
| `CAL` | Datum anzeigen (0/1) |
| `R G B` | optional: Farbe der Uhr |

!!! note "Abweichende Beschreibung"
    Die Beschreibung für das Timebox Evo lautet `45 00 01 TT XX WW EE CC RRGGBB`
    (Typ vor den Ein/Aus-Schaltern, `XX` = Zeit anzeigen). Das Layout oben ist
    das der Community-Implementierung für Pixoo-artige Geräte. Falls dein Gerät
    bei `clock` nicht richtig reagiert, probiere die Evo-Variante mit
    `pixoo raw 45 00 01 02 01 00 00 01 ff 00 00`.

## Licht (`45 01 …`) ✅

```text
45 01 R G B BRIGHT EFFECT POWER 00 00 00
```

| Byte | Bedeutung |
| --- | --- |
| `R G B` | Farbe |
| `BRIGHT` | Helligkeit 0–100 |
| `EFFECT` | `00` einfarbig; `01`–`04` Animationen (beim Evo: Liebe, Pflanzen, Mückenschutz, Schlafen) |
| `POWER` | `01` an, `00` aus |
| `00 00 00` | fest |

Das Community-„Aus“ ist `45 01 01 01 01 00 00 00 00 00 00` (Farbe fast
schwarz, Helligkeit 0, Power 0). Das ist, was `pixoo off` sendet.

## Cloud (`45 02`) ✅

Keine Optionen. Zeigt die Galerie-Animationen, die per App aufs Gerät geladen
wurden.

## Effekte (`45 03 NN`) ✅

`NN` ist die Nummer des Effekts (0-basiert, in der Reihenfolge der App).

## Visualisierung (`45 04 NN`) ✅

`NN` ist die Nummer des Equalizers/Visualizers (0-basiert, Reihenfolge der App).

## Design (`45 05`) ✅

Schaltet auf den Design-Kanal. Der Tab (Design 1–3) wird mit einem zweiten
Befehl gewählt: `BD 17 NN` (SET_DESIGN, `NN` = 0-basiert).

## Scoreboard (`45 06 00 RR RR BB BB`) ✅

| Byte | Bedeutung |
| --- | --- |
| `06 00` | Kanal Scoreboard |
| `RR RR` | Punkte rot, 0–999, little-endian |
| `BB BB` | Punkte blau, 0–999, little-endian |

## Aktuellen Kanal lesen

`46` (GET_VIEW) – Antwortformat siehe [Antworten](responses.md).
