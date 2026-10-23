# Bilder und Animationen

Das Pixoo hat **keinen** Befehl „setze Pixel (x, y)“. Wer ein einzelnes Pixel
ändern will, schickt ein komplettes 16×16-Bild. Der Client merkt sich dafür
optional die Zeichenfläche (`--state`).

## Pixelkodierung

Ein Bild besteht aus einer **Palette** und einem **Indexfeld**:

1. Alle Pixel werden zeilenweise (links→rechts, oben→unten) durchlaufen,
   Index = `x + 16 * y`.
2. Jede neu auftretende Farbe wird an die Palette angehängt, jeder Pixel
   speichert den Palettenindex.
3. Paletteneinträge sind je 3 Byte `R G B`.
4. Bits pro Pixel: `ceil(log2(Farben))`, mindestens 1. Bei 2 Farben also 1 Bit,
   bei 3–4 Farben 2 Bit, bei 5–8 Farben 3 Bit … bei 256 Farben 8 Bit.
5. Die Indizes werden **LSB-first** ohne Lücken in Bytes gepackt: der erste
   Pixel belegt die niedrigsten Bits des ersten Bytes. Das letzte Byte wird
   mit Nullen aufgefüllt.

Die Farbanzahl steht in einem Byte; bei 256 Farben wird `00` geschrieben.
Das Pixoo hat nur 256 Pixel, 256 Farben sind also das Maximum.

## Frame

Ein Frame (eine Bildzeile der Animation oder das Einzelbild) hat das Format

```text
AA | LL LL | TT TT | RR | NN | PALETTE (N×3) | PIXEL (gepackt)
```

| Feld | Bedeutung |
| --- | --- |
| `AA` | Frame-Start (fest `0xAA`) |
| `LL LL` | Länge des **gesamten** Frames inklusive `AA` und `LL LL`, little-endian |
| `TT TT` | Anzeigedauer in ms, little-endian (bei Einzelbildern `00 00`) |
| `RR` | Paletten-Reset-Flag; die Community sendet immer `00` (Palette pro Frame) |
| `NN` | Anzahl Farben (`00` = 256) |

## Einzelbild: `0x44` SET_IMAGE ✅

```text
44 | 00 0A 0A 04 | FRAME
```

Die vier Bytes `00 0A 0A 04` sind ein fester Kopf (vermutlich Hinweis auf
die Größe `0x0A0A`; bedeutungslos für 16×16, aber nötig).

### Durchgerechnetes Beispiel

Zwei rote Pixel oben links, Rest schwarz (`pixoo pixel -p 0,0,red -p 1,0,red -b black`):

```text
01                      Start
34 00                   Länge 0x34 = 52 (Befehl + Argumente + 2 Prüfsumme)
44                      SET_IMAGE
00 0A 0A 04             fester Kopf
AA                      Frame-Start
2D 00                   Framelänge 45 Byte
00 00                   Zeit 0
00                      Palette-Reset 0
02                      2 Farben
FF 00 00                Farbe 0 = rot (erster Pixel)
00 00 00                Farbe 1 = schwarz
FC FF FF … FF           256 Pixel × 1 Bit = 32 Byte
                        (erstes Byte: Pixel 0,1 = 0, Pixel 2–7 = 1 → 0b11111100)
45 22                   Prüfsumme 0x2245
02                      Ende
```

## Animation: `0x49` SET_ANIMATION_FRAME ✅

Animationen laufen als Endlosschleife. Alle Frames werden aneinandergehängt
und in **Stücke zu 200 Byte** geteilt. Jedes Stück ist ein eigener Befehl:

```text
49 | SS SS | NN | CHUNK (≤ 200 Byte)
```

| Feld | Bedeutung |
| --- | --- |
| `SS SS` | Summe der Längen aller Frames, little-endian (bei allen Stücken gleich) |
| `NN` | Laufende Nummer des Stücks, ab 0 |

Grenzen (aus dem Aufbau): Gesamtgröße < 65536 Byte, höchstens 256 Stücke
(also < 51 kB). Die Community begrenzt außerdem auf **60 Frames**, weil
das Gerät sonst Probleme macht. Der Client dünnt längere Animationen
gleichmäßig aus und addiert die Zeiten der entfernten Frames.

Für die Frames einer Animation gilt `TT TT` = Anzeigedauer in ms. Ein
Einzelbild-GIF wird als Einzelbild gesendet.

!!! tip "GIF-Hinweise der Community"
    Quell-GIFs sollten 16×16, nicht interlaced und mit globaler Palette sein.
    Der Client skaliert fremde Größen mit nearest-neighbour, aber
    runterskalierte Fotos sehen bei 16×16 selten gut aus.

## Prüfsumme bei großen Bildern

Für Bilder mit vielen verschiedenen hellen Farben überschreitet die
Prüfsumme 65535. Siehe [Überblick](overview.md);
der Client schreibt dann 4 Byte, wie hass-divoom.

## Text

Es gibt keinen Text-Befehl für das Pixoo. Lauftext entsteht, indem man den
Text als Bild rendert, pro Schritt um ein paar Pixel verschiebt und das als
Animation sendet (so macht es hass-divoom). Der Client enthält das noch
nicht – mit einer GIF-Vorlage geht es aber schon über `pixoo image`.

## Sonstige Bildbefehle 📖

| Befehl | Hinweis |
| --- | --- |
| `0x8B` SET_GIF | Variante für größere Geräte (Pixoo Max u. a.); siehe [Gerätevarianten](variants.md) |
