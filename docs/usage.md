# Benutzung (CLI)

Das Binary heißt `pixoo`. Globale Optionen (vor oder nach dem Unterbefehl):

| Option | Umgebungsvariable | Bedeutung |
| --- | --- | --- |
| `-d, --device MAC` | `PIXOO_MAC` | Bluetooth-Adresse |
| `-c, --channel N` | `PIXOO_CHANNEL` | RFCOMM-Kanal (Standard 1) |
| `--dry-run` | | Nicht verbinden, nur die Bytes ausgeben |
| `-v, --verbose` | | Gesendete und empfangene Bytes ausgeben |

## Pixel setzen

Das Gerät hat keinen Befehl „setze Pixel (x,y)“. Es kann nur ganze Bilder
anzeigen. Der Client baut deshalb aus den Pixeln ein 16×16-Bild
([Details](protocol/images.md)).

```bash
# zwei Pixel auf schwarzem Grund (x,y: 0..15, Ursprung oben links)
pixoo pixel -p 0,0,red -p 15,15,#00ff00 --background black

# Farben: #rrggbb, rrggbb, #rgb oder red, green, blue, yellow, cyan,
#         magenta, orange, purple, pink, white, black
```

### Pixel über mehrere Aufrufe hinweg behalten

Mit `--state DATEI` (oder `PIXOO_STATE`) wird die Zeichenfläche zwischen den
Aufrufen gespeichert (768 Byte RGB). So kannst du Pixel nacheinander setzen:

```bash
export PIXOO_STATE=~/.cache/pixoo.canvas
pixoo fill black
pixoo pixel -p 8,8,white
pixoo pixel -p 9,8,red     # (8,8) bleibt erhalten
pixoo show                 # Zeichenfläche erneut senden
```

!!! note
    Der Zustand liegt nur im Client. Das Gerät kann sein aktuelles Bild nicht
    zurücklesen. Wurde das Bild am Gerät oder per App geändert, stimmt die
    Datei nicht mehr mit der Anzeige überein.

## Bilder und Animationen

```bash
pixoo image logo.png            # wird auf 16×16 skaliert (nearest)
pixoo image anim.gif            # animierte GIFs laufen als Schleife
pixoo image anim.gif --time 80  # feste Frame-Zeit in ms
```

Transparente Pixel werden schwarz. Animationen sind auf 60 Frames begrenzt;
längere werden gleichmäßig ausgedünnt (die Frame-Zeiten werden aufsummiert).

## Kanäle und Anzeige

```bash
pixoo brightness 40
pixoo clock --style 1 --h24 --calendar --color '#ff8800'
pixoo light blue --brightness 60 --effect 0
pixoo on
pixoo off
pixoo channel cloud              # clock|light|cloud|effects|visualization|design|scoreboard
pixoo effect 3
pixoo visualization 2
pixoo design 1
pixoo scoreboard 3 7
```

## Zeit, Wetter, Tools

```bash
pixoo set-time                          # jetzt (lokale Zeit)
pixoo set-time 2026-10-23T19:45:00
pixoo weather 18 6                      # 18 °C, Regen
pixoo timer 1                           # 1 start, 0 stop
pixoo countdown 5 0                     # 5:00 starten
```

## Erforschen des Protokolls

```bash
pixoo raw 74 64                 # Befehl 0x74 mit Argument 0x64 (Helligkeit 100)
pixoo raw --read 46             # Befehl 0x46 senden und Antwort ausgeben
pixoo get-view
pixoo listen 30                 # 30 s lang alles mitschneiden
```

`raw` nimmt Hex (`74 64`, `0x74,0x64` oder `7464`); das erste Byte ist der
Befehl, der Rest die Argumente. Rahmen, Länge und Prüfsumme ergänzt der Client.
Die Befehlsliste steht in der [Befehlsreferenz](protocol/commands.md).
