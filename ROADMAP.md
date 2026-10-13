# Roadmap di Postcritica

**Stadio: pre-alpha** · prossimo: alpha (i flussi principali portati in Rust e usabili)

> **Ultima revisione:** 2026-10-13 · **Ultimo aggiornamento:** 2026-10-13 · **Modifica:** minor (analisi di una foto con Ollama, archivio SQLite, veste grafica della tua app) · **Obiettivo:** parità funzionale con il programma Python di riferimento, in un'app desktop nativa Rust

Riassunto di una pagina su dove siamo. Il progetto nasce come **port in Rust** del programma Python
`PhotoInsight-Critic`: l'obiettivo di questa fase è **raggiungerne la parità funzionale**.

Agenti: leggete questo, poi scegliete il lavoro da [docs/gaps.md](docs/gaps.md). Metodo ed evidenze in
[docs/target-app-parity.md](docs/target-app-parity.md); le tappe in [plan/execution-plan.md](plan/execution-plan.md).

## Numeri di testa

| | Valore | Tipo |
|---|---:|---|
| **Funzioni portate** (checklist) | **30%** di 43 righe (✅ 11, 🟡 4) | misurato: [parity-checklist.md](docs/parity-checklist.md) |
| **Tappe**: T0, T1 e il cuore di T2/T3/T7 (analisi + archivio + UI) | — | misurato: [plan/STATUS.md](plan/STATUS.md) |
| **Pronto per il lavoro reale** | **~15%** | stimato |
| Mancante per **alpha** | **~180–340 h** | stimato |
| Mancante per la **parità** col Python | **~320–640 h** | stimato |

## Cosa fa già

- **Analizza una foto** con **Ollama** in locale: punteggio, tag, punti di forza e debolezza; il report
  compare a destra ed è **salvato nell'archivio SQLite**.
- **Libreria**: si apre una cartella di foto (o una demo), si vede la griglia e, per la foto scelta,
  **metadati EXIF** e **profilo visivo**.
- **Veste grafica della tua app**: palette scura (`#0b0d10` sfondo, `#13171c` pannelli, `#5da9ff` blu).
- **Eseguibile** costruito (`dist/postcritica.exe`).

## Tappe (dal piano)

| Tappa | Contenuto | Stato |
|---|---|---|
| T0 | Fondamenta: workspace, `xtask`, git, struttura | ✅ |
| T1 | `archive`: schema SQLite, migrazioni | ✅ (import dal Python ancora da fare) |
| T2 | `metadata` + `analysis`: EXIF/XMP, profilo visivo, anteprime | 🟡 EXIF e anteprime fatti; XMP e indicatori in UI mancano |
| T3 | `ai`: provider Ollama/OpenAI, prompt, schemi, recupero JSON | 🟡 Ollama fatto; OpenAI e recupero JSON mancano |
| T4 | `engine`: analisi, punteggi, scrittura record | 🟡 analisi singola fatta; punteggi/manutenzione report mancano |
| T5 | `engine`: batch persistente, sincronizzazione cartelle | ⬜ |
| T6 | `report`: PDF, CSV, JSON, JSONL | 🟡 CSV e JSON fatti; PDF manca |
| T7 | `ui-egui`: libreria, dettaglio, analisi, impostazioni, raccolte, statistiche | 🟡 libreria, dettaglio e analisi fatti |
| T8 | Canale di controllo, parità col Python | ⬜ |

## Prossimi passi

Ordinati; dettaglio in [docs/gaps.md](docs/gaps.md).

1. **Vedere la foto analizzata insieme al report** nell'archivio (schermata «Archivio e cronologia»).
2. **Batch**: analizzare una cartella intera, con avanzamento.
3. **Tag e raccolte**, poi le **statistiche** (i grafici).

## Registro dei progressi

- **2026-10-13 (analisi + archivio):** client Ollama, prompt, report (punteggio/tag/forze/debolezze),
  archivio SQLite, pannello del report, veste grafica della tua app. Funzioni 17% → 30%.
- **2026-10-13 (fetta usabile):** griglia di miniature, ispettore con EXIF e profilo visivo, demo.
- **2026-10-13:** creati le regole, gli standard, l'architettura e il piano.

## Cronologia delle revisioni

| Data | Modifica | Riassunto |
|---|---|---|
| 2026-10-13 | minor | Analisi con Ollama, archivio SQLite e veste grafica; funzioni 17% → 30% |
| 2026-10-13 | minor | Prima fetta usabile: griglia, ispettore, demo, eseguibile; funzioni 0% → 17% |
| 2026-10-13 | major | Creata: scheletro Rust, regole, architettura e piano di migrazione |
