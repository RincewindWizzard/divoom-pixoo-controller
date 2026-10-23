# Einrichtung unter Linux

## Voraussetzungen

* Ein Bluetooth-Adapter, der von BlueZ unterstützt wird (`bluetoothctl list`).
* Rust (stable) zum Bauen, oder ein fertiges Binary von der
  [Release-Seite](https://github.com/RincewindWizzard/divoom-pixoo-controller/releases).
* Der Client braucht **keine** zusätzlichen Bibliotheken (kein libbluetooth,
  kein D-Bus): er öffnet einen RFCOMM-Socket direkt über den Kernel.

## Koppeln

Das Pixoo muss einmalig mit dem Rechner gekoppelt werden. Vorher das Gerät vom
Handy trennen bzw. die Divoom-App beenden – das Pixoo akzeptiert in der Regel
nur eine Verbindung.

```text
$ bluetoothctl
[bluetooth]# power on
[bluetooth]# agent on
[bluetooth]# scan on
  … [NEW] Device 11:22:33:44:55:66 Pixoo
[bluetooth]# scan off
[bluetooth]# pair 11:22:33:44:55:66
[bluetooth]# trust 11:22:33:44:55:66
[bluetooth]# exit
```

Die Adresse ist die Bluetooth-MAC (`PIXOO_MAC`). Je nach Modell heißt das Gerät
`Pixoo`, `Pixoo-Max`, `Ditoo-Audio` usw.

!!! note
    Es muss **nicht** per `connect` verbunden werden. Der Client baut die
    RFCOMM-Verbindung selbst auf. Ein vorheriges `bluetoothctl connect` kann
    sogar stören (Audio-Profile bei Ditoo/Timoo).

## RFCOMM-Kanal

Beim klassischen Pixoo ist es Kanal **1** (Standard im Client). Geräte mit
Audio (Ditoo, Timoo, Tivoo) verwenden oft Kanal 2, das Timebox Mini Kanal 4.
Den richtigen Kanal verrät SDP:

```bash
sdptool browse 11:22:33:44:55:66      # Paket bluez-utils / bluez-deprecated
# oder
bluetoothctl info 11:22:33:44:55:66
```

Gesucht ist der Eintrag „Serial Port“ mit `Channel: N`. Dann:

```bash
pixoo --channel 2 --device 11:22:33:44:55:66 get-view
# oder per Umgebungsvariable
export PIXOO_MAC=11:22:33:44:55:66 PIXOO_CHANNEL=2
```

## Testen

```bash
pixoo --verbose brightness 50
pixoo --verbose get-view
```

`get-view` schickt Befehl `0x46` und gibt die Antwort aus – ein guter
Verbindungstest.

## Fehlersuche

| Symptom | Ursache / Abhilfe |
| --- | --- |
| `Host is down` / `Connection refused` | Nicht gekoppelt, Gerät aus, außer Reichweite oder falscher Kanal |
| `Device or resource busy` | Handy/App oder anderer Prozess hält die Verbindung |
| `Permission denied` | Selten; RFCOMM-Sockets brauchen normalerweise keine Root-Rechte. Prüfe, ob BlueZ läuft (`systemctl status bluetooth`) |
| Verbindung steht, aber nichts passiert | Falscher RFCOMM-Kanal (siehe oben) |
| Verbindung bricht beim ersten Befehl ab | Dasselbe: anderen Kanal probieren |
| Bild ist verzerrt | Quelle nicht 16×16; der Client skaliert mit „nearest neighbour“ |
