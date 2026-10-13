# Gap: cosa manca per portare PhotoInsight Critic in Rust

> **Ultima revisione:** 2026-10-13 · **Ultimo aggiornamento:** 2026-10-13 · **Modifica:** major (creato) · **Obiettivo:** parità funzionale con il programma Python

Ogni mancanza del port, ordinata per **impatto** (quanto blocca il raggiungimento della parità), poi per
sforzo. **Gli agenti scelgono il lavoro dall'alto.** Quando chiudi un gap, accorcialo e aggiorna le righe
in [parity-checklist.md](parity-checklist.md) e i numeri in [target-app-parity.md](target-app-parity.md) e
[ROADMAP.md](../ROADMAP.md).

Le ore sono ore di orologio di un agente. `B` segna un gap che blocca **alpha**.

## Ordinati

| # | Gap | Evidenza | Impatto | Ore | Documento |
|---:|---|---|---|---:|---|
| 1 B | **Nessun archivio SQLite**: l'unico backend è in memoria | `crates/archive` ha solo `MemoryArchive`; il Python usa 11 tabelle | senza persistenza l'app non ricorda nulla | 40–70 | [architecture](architecture.md#5-modello-dati-sqlite) |
| 2 B | **Nessuna lettura EXIF/XMP/IPTC** | `metadata::read` risponde "non supportato" | l'app non sa cosa contiene una foto | 30–60 | [architecture](architecture.md#3-le-crate) |
| 3 B | **Nessun provider AI** (Ollama/OpenAI) | `crates/ai` ha solo il `trait` e un finto | l'analisi non esiste | 40–80 | [architecture](architecture.md#6-integrazione-ai) |
| 4 B | **Nessuna UI** oltre la finestra vuota | `crates/ui-egui` mostra solo un'etichetta | non c'è applicazione da usare | 80–150 | [architecture](architecture.md#3-le-crate) |
| 5 B | **Nessun `engine` oltre l'analisi minima** | `Engine` sa solo `analyze` col punteggio | mancano batch, sincronizzazione, tag, raccolte | 80–140 | [execution-plan](../plan/execution-plan.md) |
| 6 | **Nessun supporto RAW** | il Python legge CR3/RAW via `rawpy` (C) e `pyexiv2` | gli utenti RAW non possono importare | 40–120 + scelta di una crate | [architecture](architecture.md#3-le-crate) |
| 7 | **Nessun PDF** | `photoinsight-report` fa CSV e JSON | manca il report stampabile | 20–40 | [execution-plan](../plan/execution-plan.md) |
| 8 | **Nessun canale di controllo** | — | gli agenti non possono pilotare l'app | 10–20 | [AGENTS.md](../AGENTS.md) |
| 9 | **Strumento di import dal database Python** | decisione presa: schema nuovo + import; l'import non esiste ancora | chi migra perderebbe l'archivio | 15–30 | [execution-plan](../plan/execution-plan.md) |
| 10 | **Nessun test end-to-end** | i test coprono solo le crate isolate | le regressioni arriverebbero all'utente | 15–30 | [standards/progress-docs](../standards/progress-docs.md) |

I gap che bloccano alpha (`B`) sommano **~270–500 h**.

## Gap per tipo

- **Funzioni:** 1, 2, 3, 5, 6, 7
- **UI / UX:** 4
- **Piattaforme e strumenti:** 8
- **Dati e compatibilità:** 9
- **Qualità:** 10

## Chiusi di recente

- 2026-10-13: creato lo scheletro (workspace, `xtask`, 8 crate, 22 test).

## Cronologia delle revisioni

| Data | Modifica | Riassunto |
|---|---|---|
| 2026-10-13 | major | Creato dal programma Python, ordinato per la migrazione a Rust |
