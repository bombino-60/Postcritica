# Roadmap di Postcritica

**Stadio: pre-alpha** · prossimo: alpha (i flussi principali portati in Rust e usabili)

> **Ultima revisione:** 2026-10-13 · **Ultimo aggiornamento:** 2026-10-13 · **Modifica:** minor (prima fetta usabile: griglia, ispettore EXIF, demo, eseguibile) · **Obiettivo:** parità funzionale con il programma Python di riferimento, in un'app desktop nativa Rust

Riassunto di una pagina su dove siamo. Il progetto nasce come **port in Rust** del programma Python
`PhotoInsight-Critic`: l'obiettivo di questa fase è **raggiungerne la parità funzionale** con una struttura
stratificata e testabile.

Agenti: leggete questo, poi scegliete il lavoro da [docs/gaps.md](docs/gaps.md). Metodo ed evidenze in
[docs/target-app-parity.md](docs/target-app-parity.md); le tappe in [plan/execution-plan.md](plan/execution-plan.md).

## Numeri di testa

| | Valore | Tipo |
|---|---:|---|
| **Funzioni portate** (checklist) | **17%** di 43 righe (✅ 5, 🟡 5) | misurato: [parity-checklist.md](docs/parity-checklist.md) |
| **Tappe**: T0 chiusa; fetta verticale (parte di T2/T7) usabile | — | misurato: [plan/STATUS.md](plan/STATUS.md) |
| **Pronto per il lavoro reale** | **~5%** | stimato |
| Mancante per **alpha** | **~250–470 h** | stimato (archivio + AI + UI completa) |
| Mancante per la **parità** col Python | **~380–760 h** | stimato |

## Cosa è già in piedi

- Workspace Cargo con 8 crate stratificati (L0–L5) + il binario `postcritica`.
- Gate di qualità: `cargo xtask ci` (fmt, clippy, test, strati, asset, documenti) — verde.
- **Un'app usabile:** si apre una cartella di foto (o una demo), si vede la griglia delle miniature, e
  selezionandone una i **metadati EXIF** e il **profilo visivo** (luminosità, contrasto).
- **Eseguibile** costruito (`dist/postcritica.exe`).
- Regole e standard in italiano ([AGENTS.md](AGENTS.md), [standards/](standards/)).

## Tappe (dal piano)

| Tappa | Contenuto | Stato |
|---|---|---|
| T0 | Fondamenta: workspace, `xtask`, git, struttura | ✅ |
| T1 | `archive`: schema SQLite, migrazioni, import dal Python | ⬜ |
| T2 | `metadata` + `analysis`: EXIF/XMP, profilo visivo, anteprime | 🟡 EXIF e anteprime fatti; XMP e indicatori in UI mancano |
| T3 | `ai`: provider Ollama/OpenAI, prompt, schemi, recupero JSON | ⬜ |
| T4 | `engine`: analisi, punteggi, scrittura record | ⬜ |
| T5 | `engine`: batch persistente, sincronizzazione cartelle | ⬜ |
| T6 | `report`: PDF, CSV, JSON, JSONL | 🟡 CSV e JSON fatti |
| T7 | `ui-egui`: libreria, dettaglio, analisi, impostazioni, raccolte, statistiche | 🟡 griglia e ispettore fatti |
| T8 | Canale di controllo, parità col Python | ⬜ |

## Prossimi passi

Ordinati; dettaglio in [docs/gaps.md](docs/gaps.md).

1. **T1 — `archive`**: schema SQLite nuovo, migrazioni, e lo strumento di import dal database Python.
2. **T2 (resto)**: XMP (rating, keyword), indicatori tecnici nella UI, crop per il prompt.
3. **T3 — `ai`**: i provider Ollama/OpenAI con prompt e recupero JSON.

## Registro dei progressi

- **2026-10-13 (fetta usabile):** griglia di miniature, ispettore con EXIF e profilo visivo, pulsante
  Demo; `raster::decode_thumbnail` e `metadata::read` reali; eseguibile release. 26 test verdi.
- **2026-10-13:** creati le regole, gli standard, l'architettura e il piano; workspace Rust con 8 crate.

## Cronologia delle revisioni

| Data | Modifica | Riassunto |
|---|---|---|
| 2026-10-13 | minor | Prima fetta usabile: griglia, ispettore, demo, eseguibile; funzioni 0% → 17% |
| 2026-10-13 | major | Creata: scheletro Rust, regole, architettura e piano di migrazione |
