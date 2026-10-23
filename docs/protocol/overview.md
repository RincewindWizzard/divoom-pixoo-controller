# Protokoll-Überblick

!!! info "Herkunft"
    Inoffiziell, aus Community-Reverse-Engineering
    ([Quellen](../references.md)). Status-Markierungen auf den Folgeseiten:
    **implementiert** = vom Client gesendet und per Unit-Test gegen das
    dokumentierte Format geprüft; **dokumentiert** = nur aus Quellen
    übernommen. Nichts davon wurde vom Autor an echter Hardware verifiziert.

## Transport

| Eigenschaft | Wert |
| --- | --- |
| Funktechnik | Bluetooth **Classic** (BR/EDR), kein BLE |
| Profil | SPP (Serial Port Profile), UUID `0x1101` |
| Protokoll | RFCOMM |
| Kanal | meist **1** (Ditoo/Timoo/Tivoo: oft 2, Timebox Mini: 4) – per SDP abfragbar |
| Kopplung | normale Bluetooth-Kopplung, keine PIN nötig (Just Works) |
| Verbindungen | in der Regel nur eine gleichzeitig (Handy-App trennen!) |

Nach dem Verbinden schickt man Nachrichten als Bytestrom. Die Community-Clients
warten nach dem Connect ca. 0,5 s, bevor sie senden.

## Nachrichtenformat (Host → Gerät)

```text
 01 | LL LL | CMD | ARGS… | CS CS | 02
 │    │       │      │       │      └─ Ende (0x02)
 │    │       │      │       └──────── Prüfsumme, little-endian
 │    │       │      └──────────────── 0..n Argumentbytes
 │    │       └─────────────────────── Befehlsbyte
 │    └─────────────────────────────── Länge, little-endian
 └──────────────────────────────────── Start (0x01)
```

* **Länge** `LL LL` = `1 (CMD) + len(ARGS) + 2 (Prüfsumme)` = `len(ARGS) + 3`.
* **Prüfsumme** = Summe aller Bytes von `LL` bis zum letzten Argumentbyte
  (also *ohne* Start, Prüfsumme und Ende), als 16-Bit little-endian.
  Wird die Summe ≥ 65535 (das passiert bei Bildern mit vielen hellen Farben),
  schreiben die Community-Clients sie als **32-Bit** little-endian, ohne die
  Länge anzupassen. Der Client macht das genauso. Ob ein klassisches Pixoo
  das erwartet oder die Summe modulo 65536 nimmt, ist nicht abschließend
  geklärt (siehe [Gerätevarianten](variants.md)).
* Mehrbyte-Zahlen sind überall **little-endian** (niederwertiges Byte zuerst).

### Beispiele

Helligkeit 100 % (`0x74`, Argument `0x64`):

```text
01 04 00 74 64 DC 00 02
   └len=4  │  │  └─ Summe 04+00+74+64 = 0xDC → "DC 00"
          cmd arg
```

Aktuelle Ansicht abfragen (`0x46`, keine Argumente):

```text
01 03 00 46 49 00 02
```

## Escaping (alte Firmware)

Bei sehr alten Timebox-Firmwares dürfen die Bytes `01`, `02` und `03` nicht
roh in Länge/Payload/Prüfsumme vorkommen. Sie werden dann als `03 04`, `03 05`
bzw. `03 06` gesendet (`03` gefolgt von `Byte + 3`). Neuere Geräte – darunter
das Pixoo – brauchen das nicht mehr, und die Community-Clients schalten es
standardmäßig ab. Der Client implementiert kein Escaping.

## Segmentierung

Ältere Beschreibungen (Timebox Evo) teilen lange Nachrichten in Stücke von
666 Byte. Die aktuellen Clients senden jede Nachricht in einem Rutsch
(`sendall`). Bilder passen mit höchstens ~1,6 kB in eine Nachricht, und
Animationen werden vom Protokoll selbst in 200-Byte-Stücke geteilt
([Bilder](images.md)).

## Antworten (Gerät → Host)

Das Gerät antwortet auf manche Befehle und sendet teils unaufgefordert
Nachrichten (z. B. bei Tastendruck am Gerät). Format und bekannte Inhalte:
[Antworten](responses.md).

## Lebenszeichen („Ping“)

Es gibt keinen echten Ping. Die Community nutzt `GET_VIEW` (`0x46`) als
Verbindungstest: kommt irgendeine Antwort, lebt die Verbindung.
