# mybot

`mybot` ist ein macOS/Linux CLI-Tool (Rust) zum Scannen, Planen und Ausführen von Media-Renames.

## Architektur

Das Projekt ist modular aufgebaut:

- `scanner`: liest Dateien aus einem Pfad (optional rekursiv)
- `parser`: erkennt Movies (`title + year`) und Episoden (`SxxEyy` / `x`-Pattern)
- `template`: rendert Zielnamen mit Platzhaltern `{title}`, `{year}`, `{season:02}`, `{episode:02}`, `{ext}`
- `planner`: erstellt Operationen (`from -> to`) inkl. Conflict/Unknown-Report
- `executor`: führt `rename|move|copy` aus (oder dry-run)
- `matcher`: TMDB-Movie-Matching über austauschbaren HTTP-Client-Trait

## CLI Usage

```bash
mybot scan <path> [--recursive]
mybot plan <path> [--recursive] --format "<template>" [--action rename|move|copy] [--output <dir>] [--dry-run] [--db tmdb --tmdb-key <key>]
mybot apply <path> [--recursive] --format "<template>" [--action rename|move|copy] [--output <dir>] [--dry-run] [--db tmdb --tmdb-key <key>]
mybot match movie <path> --tmdb-key <key> [--language de-DE] [--dry-run]
```

### Beispiele

```bash
mybot scan ./media --recursive
mybot plan ./media --recursive --format "{title} ({year}).{ext}" --dry-run
mybot apply ./media --format "{title} - S{season:02}E{episode:02}.{ext}" --action rename
mybot match movie ./movies --tmdb-key "$TMDB_KEY"
mybot plan ./movies --format "{title} ({year}).{ext}" --db tmdb --tmdb-key "$TMDB_KEY" --dry-run
```

## TMDB Key

TMDB-Key kann per Flag oder Umgebungsvariable übergeben werden:

- `--tmdb-key <key>`
- `TMDB_KEY=<key>`

Es werden **keine Secrets im Repo** gespeichert.

## Testen

Alle Tests lokal und ohne echtes Internet:

```bash
cargo test
```

- Parser Unit-Tests: 30+ Fälle
- Template Unit-Tests: 15+ Fälle
- Matching Unit-Tests: Mock-HTTP-Client + JSON Fixtures
- Integrationstest: `plan/apply --dry-run` in Tempdir
