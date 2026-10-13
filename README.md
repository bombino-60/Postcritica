# Postcritica

Applicazione **desktop nativa in Rust** per l'analisi tecnica, la catalogazione e la critica fotografica,
con archivio locale SQLite e analisi tramite **Ollama** (modelli vision in locale) o **ChatGPT Vision**.

È il **port in Rust** del programma Python esistente
(`D:\Programmi Sviluppo\Unificato\postcritica-Critic`), che resta il riferimento funzionale. La lingua di
lavoro del codice è l'italiano (commenti, docstring e stringhe); gli identificatori sono in inglese.

## Stato

**Pre-alpha**, allo scheletro. Vedi [`ROADMAP.md`](ROADMAP.md) e [`plan/STATUS.md`](plan/STATUS.md).

## Primi passi

```powershell
# compila e testa le crate di base
cargo test --workspace

# tutti i gate (formattazione, lint, test, strati, asset, documenti)
cargo xtask ci

# avvia l'applicazione desktop
cargo run -p postcritica
```

## Documenti

| File | A cosa serve |
|---|---|
| [`AGENTS.md`](AGENTS.md) | Come si lavora: le regole del progetto. |
| [`ROADMAP.md`](ROADMAP.md) | Dove siamo, in una pagina. |
| [`docs/architecture.md`](docs/architecture.md) | Com'è fatto il programma e come si arriva dal Python. |
| [`docs/gaps.md`](docs/gaps.md) | Cosa manca, ordinato per impatto. |
| [`docs/parity-checklist.md`](docs/parity-checklist.md) | Una riga per funzione. |
| [`plan/execution-plan.md`](plan/execution-plan.md) | Le tappe della migrazione. |
| [`standards/`](standards/) | Gli standard del progetto (mai-in-crash, documenti, font, corpora). |

## Struttura

```
crates/    le crate stratificate (raster, metadata, analysis, archive, ai, report, engine, ui-egui)
apps/      il binario desktop (postcritica)
xtask/     il tooling (ci, layers, assets, parity)
docs/      i documenti di avanzamento e l'architettura
standards/ gli standard del progetto
plan/      il piano e lo stato del lavoro (STATUS, execution-plan)
assets/    gli asset e la loro attribuzione
```

## Licenza

Rilasciato sotto **MIT OR Apache-2.0** (a tua scelta): vedi [`LICENSE-MIT`](LICENSE-MIT) e
[`LICENSE-APACHE`](LICENSE-APACHE). Attribuzioni in [`NOTICE`](NOTICE) e
[`assets/ATTRIBUTION.md`](assets/ATTRIBUTION.md).
