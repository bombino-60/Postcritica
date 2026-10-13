# Stato

**Tappa corrente:** T0 chiusa; in corso una **fetta verticale usabile** (parte di T2 e T7).
**Prossimo task:** chiudere T1 (`archive`, SQLite + import dal Python), poi completare T2.

## Fatto
- Regole e standard in italiano (`AGENTS.md`, `standards/`), tarati sul Rust.
- Architettura di destinazione e mappa dal Python (`docs/architecture.md`); piano a tappe; documenti.
- Workspace Cargo con 8 crate stratificati (L0–L5) + il binario `postcritica`; `xtask` con
  `ci`/`layers`/`assets`/`parity`.
- Licenza MIT OR Apache-2.0; repository git collegato al remote `origin`.
- **Fetta usabile:** finestra egui con **griglia di miniature**, **ispettore** (metadati EXIF + profilo
  visivo) e pulsante **Demo**; lettura di una cartella in un thread separato.
- `raster::decode_thumbnail` (decodifica con tetto di memoria) e `metadata::read` (EXIF con
  `kamadak-exif`) reali.
- **26 test verdi**; `cargo xtask ci` verde; **eseguibile release** in `dist/postcritica.exe`.

## In corso
- Nulla di aperto: la fetta è chiusa e committata.

## Blocchi
- Nessuno.

## Assunzioni registrate
- Interfaccia: app desktop nativa egui/eframe.
- Codice: commenti/stringhe in italiano, identificatori in inglese.
- Provider AI: Ollama e ChatGPT/OpenAI (non ancora collegati).
- SQLite è l'unica dipendenza nativa ammessa.
- Obiettivo di fase: **parità funzionale** col Python.
- Dati: **schema nuovo** + **strumento di import** dal Python (T1).

## Metriche
- Test: 26 verdi.
- Eseguibile release: ~8 MB.
- Compilazione del workspace: OK.

## Decisioni del proprietario in sospeso
- Se introdurre il canale di controllo e MCP (tappa T8).
- Nome delle crate: al momento `postcritica-*`.
