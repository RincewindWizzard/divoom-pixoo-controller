# Gerätevarianten

Die Divoom-Geräte teilen sich im Kern dasselbe Protokoll, unterscheiden sich
aber in Details. Zusammenfassung aus den Community-Quellen (`?` = nicht belegt):

| Gerät | Anzeige | RFCOMM-Kanal | Besonderheiten |
| --- | --- | --- | --- |
| **Pixoo** | 16×16 | 1 | Referenzgerät für diesen Client |
| Pixoo Max | 32×32 | 1 | Größere Header, siehe unten |
| Ditoo / Ditoo Mic | 16×16 | 2 | Lautsprecher, Tastatur-LEDs, Scoreboard über `SET_TOOL` |
| Timoo, Tivoo | ? | 2 | Lautsprecher |
| Timebox Evo | ? | ? | Quelle des ersten Protokolls; Uhr-Befehl leicht anders |
| Timebox Mini | ? | 4 | stark eingeschränkt |
| Aurabox | ? | 4 | 8 Farben im Licht-Kanal |
| Backpack | ? | ? | Signal-Kanal (Blinker) |

## Pixoo Max (32×32)

Abweichungen laut hass-divoom:

* Bild-/Animations-Kopf: Gesamtlänge **4 Byte**, Chunk-Nummer **2 Byte**
  (beim 16×16: 2 bzw. 1 Byte).
* Vor einer Animation zwei Leerframes mit den Flags `05` und `06`; jeder Frame
  trägt das Palette-Flag `03` und die Farbanzahl als 2 Byte.
* Prüfsumme wird bei Bedarf 4 Byte lang.
* 16×16-Bilder kann das Gerät selbst hochskalieren.

Der Client unterstützt **nur 16×16**.

## Offene Fragen

* **Prüfsumme ≥ 65535** beim klassischen Pixoo: 4-Byte-Summe (wie hass-divoom)
  oder modulo 65536? Falls farbenreiche Bilder nicht erscheinen, bei diesem
  Punkt anfangen.
* **Antworten des Pixoo**: Format ist belegt (Evo), Inhalte für das Pixoo nicht.
* **Uhr-Befehl**: Layout `45 00 H24 STYLE 01 W T C [RGB]` (Pixoo) vs.
  `45 00 01 TT XX W T C RGB` (Evo) – siehe [Kanäle](channels.md).
* **Weitere SET_VIEW-Unterkanäle** (z. B. Ditoo-Lyrics `45 06 00 00 00`).
* **Paletten-Reset-Flag** `RR`: Bedeutung ungeklärt; `00` funktioniert.
