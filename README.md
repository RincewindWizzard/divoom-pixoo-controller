# Divoom Pixoo Controller

Rust-Client (CLI + Bibliothek) für das **Divoom Pixoo** (16×16) über Bluetooth
Classic unter Linux – inklusive Dokumentation des Bluetooth-Protokolls.

📖 **Dokumentation:** <https://rincewindwizzard.github.io/divoom-pixoo-controller/>

```bash
cargo build --release
export PIXOO_MAC=11:22:33:44:55:66      # vorher koppeln, siehe Doku
pixoo pixel -p 0,0,red -p 15,15,#00ff00 --background black
pixoo image anim.gif
pixoo brightness 50
pixoo --dry-run clock --style 1         # nur die Bytes anzeigen
```

> **Status:** Das Protokoll stammt aus Community-Reverse-Engineering, die
> Kodierung ist per Unit-Tests abgesichert; der Client wurde noch nicht an
> echter Hardware getestet. Fehlerberichte dazu sind willkommen.

Releases (Linux x86_64/aarch64) entstehen automatisch beim Setzen eines Tags
`vX.Y.Z`. Siehe [Entwicklung](docs/development.md).
