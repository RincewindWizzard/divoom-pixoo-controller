# Antworten des Geräts

## Format

```text
01 | LL LL | 04 | CMD | 55 | DATEN… | CS CS | 02
```

* `LL LL`, `CS CS`: wie bei ausgehenden Nachrichten (die Länge zählt alles
  zwischen `LL LL` und `CS CS` plus 2).
* `04 CMD 55`: fester Rahmen, `CMD` ist der Befehl, auf den geantwortet wird.
* `DATEN`: befehlsabhängig.

`pixoo` parst solche Nachrichten (`protocol::parse_messages`), prüft die
Prüfsumme und zeigt `CMD` und `DATEN` an. Zeilen ohne diesen Rahmen werden roh
ausgegeben.

!!! note
    Das Pixoo antwortet nicht auf jeden Befehl. Die meisten
    „Setze…“-Befehle bleiben still; die Community-Clients lesen nur zum
    Debuggen. In den Quellen sind Antworten nur für die Befehle unten
    beschrieben (Timebox Evo). Wie das Pixoo konkret antwortet, ist zu prüfen:
    `pixoo get-view` und `pixoo listen 60`.

## Bekannte Antworten (aus dem Timebox-Evo-Protokoll)

| Antwort | Beispiel-Daten | Bedeutung |
| --- | --- | --- |
| `04 46 55 CC …` | `CC` Kanal, danach Konfiguration; Helligkeit taucht darin zweimal auf | Auf `GET_VIEW` (`0x46`): aktueller Kanal |
| `04 44 55 CC` / `04 45 55 CC` | `CC` Kanal | Hinweis, dass am Gerät ein Kanal gewählt wurde |
| `04 31 55 BB` / `04 32 55 BB` | `BB` Helligkeit 0–100 | Helligkeit wurde am Gerät geändert |

Beispiel `GET_VIEW`-Antwort (Evo): `04 46 55 CC 00004A007F BB 7F010B BB 017F007F00010001000100`.

## Unaufgeforderte Nachrichten

Tastendrücke am Gerät führen zu Nachrichten wie den `44`/`45`/`31`/`32`-Antworten
oben. Mit `pixoo listen 60` und Tastendrücken kann man das mitschneiden.
Dokumentiere neue Beobachtungen bitte hier.
