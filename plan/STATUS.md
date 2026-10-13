# Stato

**Tappa corrente:** T0 — Fondamenta (chiusa)
**Prossimo task:** aprire la tappa **T1** (`archive`): schema SQLite nuovo, migrazioni, e lo strumento di import dal Python.

## Fatto
- Regole e standard scritti e adattati al Rust (`AGENTS.md`, `standards/`), in italiano.
- Architettura di destinazione e mappa dalla struttura Python (`docs/architecture.md`).
- Piano a tappe (`plan/execution-plan.md`); documenti di avanzamento (`ROADMAP.md`, `docs/`).
- Workspace Cargo con 8 crate stratificati (L0–L5) + il binario `photoinsight`.
- `xtask` con `ci`, `layers`, `assets`, `parity`; alias `cargo xtask`.
- API minime ma reali con test: **22 test verdi** (raster, metadata, analysis, archive, ai, report, engine).
- **`cargo xtask ci` verde**: fmt, clippy `-D warnings`, test, strati, asset, documenti.
- L'app egui compila (eframe 0.36.2) e apre una finestra con un'etichetta.
- Licenza **MIT OR Apache-2.0** (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`).
- Repository **git inizializzato** e primo commit (nessun remote).

## In corso
- Tappa T1 (`archive`): schema SQLite nuovo, migrazioni, e lo strumento di import dal database Python.

## Blocchi
- Nessuno.

## Assunzioni registrate
- Interfaccia: app desktop nativa egui/eframe.
- Codice: commenti/stringhe in italiano, identificatori in inglese.
- Provider AI: Ollama e ChatGPT/OpenAI, come il programma Python.
- SQLite è l'unica dipendenza nativa ammessa (l'archivio del programma è SQLite).
- L'obiettivo di questa fase è la **parità funzionale** col Python, non superarlo.
- Licenza: MIT OR Apache-2.0 (deciso dal proprietario).
- Dati: **schema nuovo** + **strumento di import** dal database Python (deciso dal proprietario).

## Metriche
- Test: 22 verdi.
- Righe Rust: ~1.100 (inclusi i test e i commenti).
- Compilazione del workspace: OK.

## Decisioni del proprietario in sospeso
- Nome definitivo del prodotto e delle crate (qui: `postcritica-*`).
- Se introdurre il canale di controllo e MCP (tappa T8).
