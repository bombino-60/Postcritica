# Roadmap di Postcritica

**Stadio: pre-alpha** · prossimo: alpha (i flussi principali portati in Rust e usabili)

> **Ultima revisione:** 2026-10-13 · **Ultimo aggiornamento:** 2026-10-13 · **Modifica:** major (creata) · **Obiettivo:** parità funzionale con il programma Python di riferimento, in un'app desktop nativa Rust

Riassunto di una pagina su dove siamo. Il progetto nasce come **port in Rust** del programma Python
`postcritica-Critic`: l'obiettivo di questa fase non è superare il Python, ma **raggiungerne la parità
funzionale** con una struttura stratificata e testabile. Poi si riparte da lì.

Agenti: leggete questo, poi scegliete il lavoro da [docs/gaps.md](docs/gaps.md). Metodo ed evidenze in
[docs/target-app-parity.md](docs/target-app-parity.md); le tappe in [plan/execution-plan.md](plan/execution-plan.md).

## Numeri di testa

| | Valore | Tipo |
|---|---:|---|
| **Funzioni portate** (checklist) | **0%** (scheletro) | misurato: righe ✅ di [parity-checklist.md](docs/parity-checklist.md) |
| **Tappe completate** | **T0 in corso** | misurato: [plan/STATUS.md](plan/STATUS.md) |
| **Pronto per il lavoro reale** | **~0%** | stimato |
| Mancante per **alpha** | **~180–360 h** | stimato (port di analisi + archivio + UI minima) |
| Mancante per la **parità** col Python | **~400–800 h** | stimato |

## Cosa è già in piedi

- Workspace Cargo con 8 crate + l'app, stratificato (L0–L5).
- Gate di qualità funzionanti: `cargo xtask ci` (fmt, clippy, test, strati, asset, parità).
- Le regole di lavoro (`AGENTS.md`) e gli standard (`standards/`) in italiano, tarati sul Rust.
- 22 test verdi sulle crate di base.

## Tappe (dal piano)

| Tappa | Contenuto | Stato |
|---|---|---|
| T0 | Fondamenta: workspace, `xtask`, git, struttura | **in corso** |
| T1 | `archive`: schema SQLite, migrazioni, query | ⬜ |
| T2 | `metadata` + `analysis`: EXIF/XMP, profilo visivo, anteprime | ⬜ |
| T3 | `ai`: provider Ollama/OpenAI, prompt, schemi, recupero JSON | ⬜ |
| T4 | `engine`: analisi singola, punteggi, scrittura record | ⬜ |
| T5 | `engine`: batch persistente, sincronizzazione cartelle | ⬜ |
| T6 | `report`: PDF, CSV, JSON, JSONL | ⬜ |
| T7 | `ui-egui`: libreria, dettaglio, analisi, impostazioni, raccolte, statistiche | ⬜ |
| T8 | Canale di controllo, parità col Python | ⬜ |

## Prossimi passi

Ordinati; dettaglio in [docs/gaps.md](docs/gaps.md).

1. Chiudere T0: `git init`, primo commit, conferma delle versioni delle dipendenze.
2. T1: portare lo schema e l'archivio SQLite, con i test sui round-trip.
3. T2: leggere EXIF/XMP e calcolare il profilo visivo in Rust.

## Registro dei progressi

- **2026-10-13:** creati le regole, gli standard, l'architettura e il piano; workspace Rust con 8 crate,
  `xtask` e 22 test verdi.

## Cronologia delle revisioni

| Data | Modifica | Riassunto |
|---|---|---|
| 2026-10-13 | major | Creata: scheletro Rust, regole, architettura e piano di migrazione |
