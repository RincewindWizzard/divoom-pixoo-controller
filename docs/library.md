# Rust-Bibliothek

Das Crate stellt neben dem Binary `pixoo` die Bibliothek `pixoo` bereit.

```rust
use pixoo::canvas::Canvas;
use pixoo::device::Pixoo;
use pixoo::rfcomm::RfcommStream;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let stream = RfcommStream::connect("11:22:33:44:55:66", 1, Duration::from_secs(10))?;
    let mut pixoo = Pixoo::new(stream);

    let mut canvas = Canvas::default();          // schwarz
    canvas.set(0, 0, [255, 0, 0])?;              // oben links rot
    canvas.set(15, 15, [0, 255, 0])?;            // unten rechts grün
    pixoo.set_image(&canvas)?;
    pixoo.set_brightness(60)?;
    Ok(())
}
```

## Module

| Modul | Inhalt |
| --- | --- |
| `pixoo::protocol` | Nachrichtenrahmen (`build_message`), Prüfsumme, Parser für Antworten, Befehlskonstanten (`cmd::*`) |
| `pixoo::canvas` | `Canvas` (16×16 RGB), Palette-/Bit-Packing, `parse_color` |
| `pixoo::device` | `Pixoo<T>` mit den Befehlen; `Transport`-Trait; `DryRun` zum Testen |
| `pixoo::rfcomm` | `RfcommStream`, ein minimaler RFCOMM-Client über `libc` |

`Pixoo<T>` ist über jeden `Read + Write`-Strom generisch. So lassen sich
Befehle ohne Hardware testen (`DryRun`) oder über andere Transporte schicken
(z. B. ein TCP-Proxy zu einem ESP32).

Eigene, noch nicht umgesetzte Befehle gehen über `Pixoo::send(cmd, &args)`.
