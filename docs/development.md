# Entwicklung und Release

## Bauen und testen

```bash
cargo build --release          # target/release/pixoo
cargo test                     # Unit-Tests für Rahmen, Prüfsumme, Bildkodierung
cargo clippy --all-targets
cargo fmt --check
```

Die Tests brauchen keine Hardware. `--dry-run` zeigt die gesendeten Bytes.

## Projektaufbau

```text
src/protocol.rs   Nachrichtenrahmen, Prüfsumme, Antwort-Parser, Befehlskonstanten
src/canvas.rs     16×16-Zeichenfläche, Palette + Bit-Packing
src/device.rs     Befehle (Pixoo<T>), Animationen, DryRun-Transport
src/rfcomm.rs     RFCOMM-Socket über libc
src/main.rs       CLI (clap)
docs/             Diese Dokumentation (MkDocs Material)
```

## Doku lokal ansehen

```bash
pip install mkdocs-material
mkdocs serve        # http://127.0.0.1:8000
```

## GitHub Pages

Der Workflow `.github/workflows/docs.yml` baut die Doku bei jedem Push auf
`main`, der `docs/**` oder `mkdocs.yml` ändert, und veröffentlicht sie auf
GitHub Pages.

!!! important "Einmalige Einstellung"
    Im Repository unter **Settings → Pages → Build and deployment → Source**
    muss **GitHub Actions** ausgewählt sein. Sonst schlägt das Deployment fehl.

## Release erstellen

Releases entstehen automatisch beim Setzen eines Tags `vX.Y.Z`:

```bash
# Version in Cargo.toml anpassen, committen, dann:
git tag v0.1.0
git push origin v0.1.0
```

Der Workflow `.github/workflows/release.yml` prüft, dass das Tag zur Version
in `Cargo.toml` passt, führt die Tests aus, baut Release-Binaries
(Linux x86_64 und aarch64) und hängt sie mit Prüfsummen an ein neues
GitHub Release. Die Release Notes werden automatisch aus den Commits erzeugt.

Der Client läuft nur unter Linux (BlueZ-RFCOMM-Sockets).
