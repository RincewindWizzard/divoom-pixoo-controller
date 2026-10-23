# Divoom Pixoo Controller

Ein Rust-Client (CLI und Bibliothek), um ein **Divoom Pixoo** (16×16 Pixel) per
Bluetooth von einem Linux-Rechner aus zu steuern – plus eine möglichst
vollständige Dokumentation des Bluetooth-Protokolls, weil es dafür kaum
offizielle Unterlagen gibt.

!!! warning "Stand der Verifikation"
    Divoom veröffentlicht das Protokoll nicht. Alles hier basiert auf dem
    Reverse Engineering der Community (siehe [Quellen](references.md)).
    Die Kodierung (Rahmen, Prüfsumme, Bildformat) ist durch Unit-Tests gegen
    die dokumentierten Formate abgesichert, **der Client wurde aber noch nicht
    gegen echte Hardware getestet**. Jede Seite im Protokollteil markiert, was
    der Client implementiert und was nur dokumentiert ist. Wenn etwas bei
    deinem Gerät anders ist: `pixoo raw` und `pixoo listen` helfen beim
    Nachforschen, und Korrekturen an der Doku sind willkommen.

## Was geht?

* Einzelne Pixel setzen, Fläche füllen, Bilder/GIFs anzeigen (16×16)
* Helligkeit, Uhr, Licht-Kanal, Effekte, Visualisierung, Design, Scoreboard
* Gerätezeit und Wetter setzen, Timer/Countdown/Lärmmesser
* Rohe Befehle senden und Antworten mitschneiden (zum Erforschen)

## Schnellstart

```bash
# 1. Gerät einmalig koppeln (siehe Einrichtung)
bluetoothctl            # scan on, pair <MAC>, trust <MAC>

# 2. Bauen
cargo build --release

# 3. Pixel setzen
export PIXOO_MAC=11:22:33:44:55:66
pixoo pixel -p 0,0,red -p 15,15,#00ff00 --background black
```

Ohne Gerät kannst du jederzeit mit `--dry-run` die Bytes ansehen, die gesendet
würden.

## Überblick über das Protokoll

Das Pixoo ist ein Bluetooth-**Classic**-Gerät (kein BLE). Es meldet einen
seriellen Dienst (SPP) auf RFCOMM-Kanal 1. Darüber werden Nachrichten der Form

```text
01 | LL LL | CMD | ARGS… | CS CS | 02
```

ausgetauscht. Details: [Protokoll-Überblick](protocol/overview.md).
