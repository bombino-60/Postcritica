# Roadmap di Postcritica

**Stadio: pre-alpha** · prossimo: alpha (i flussi principali portati in Rust e usabili)

> **Ultima revisione:** 2026-10-13 · **Ultimo aggiornamento:** 2026-10-13 · **Modifica:** minor (analisi con Ollama e ChatGPT, archivio, statistiche, raccolte, setup, batch, PDF, import dal Python) · **Obiettivo:** parità funzionale con il programma Python di riferimento

Riassunto di una pagina su dove siamo. Il progetto nasce come **port in Rust** del programma Python
`PhotoInsight-Critic`: l'obiettivo di questa fase è **raggiungerne la parità funzionale**.

Agenti: leggete questo, poi scegliete il lavoro da [docs/gaps.md](docs/gaps.md).

## Numeri di testa

| | Valore | Tipo |
|---|---:|---|
| **Funzioni portate** (checklist) | **55%** di 43 righe (✅ 20, 🟡 7) | misurato: [parity-checklist.md](docs/parity-checklist.md) |
| **Pronto per il lavoro reale** | **~40%** | stimato |
| Mancante per **alpha** | **~60–120 h** | stimato |
| Mancante per la **parità** col Python | **~150–300 h** | stimato |

## Cosa fa

- **Analizza una foto** con **Ollama** (locale) o **ChatGPT** (con una chiave API): punteggio, tag, punti
  di forza e debolezza; il report si vede a destra ed è **salvato nell'archivio SQLite**.
- **Batch**: «Analizza tutte» analizza la cartella una foto per volta, con barra di avanzamento.
- **Libreria**: griglia raggruppata per data, metadati EXIF e profilo visivo.
- **Statistiche**: distribuzione dei punteggi, fotocamere, ISO, tag più frequenti.
- **Raccolte per tag**, **valutazione a stelle**, **esporta PDF**, **importa dal database Python**.
- **Setup**: provider, modelli e chiave.
- Veste grafica con i colori e le misure del **riferimento LightCraft**.

## Tappe (dal piano)

| Tappa | Contenuto | Stato |
|---|---|---|
| T0 | Fondamenta: workspace, `xtask`, git, struttura | ✅ |
| T1 | `archive`: schema SQLite, migrazioni, import dal Python | ✅ |
| T2 | `metadata` + `analysis`: EXIF/XMP, profilo visivo, anteprime | 🟡 mancano XMP e indicatori in UI |
| T3 | `ai`: provider Ollama/OpenAI, prompt, schemi | ✅ (manca il recupero JSON avanzato) |
| T4 | `engine`: analisi, punteggi | 🟡 analisi fatta; mancano i punteggi pesati |
| T5 | `engine`: batch, sincronizzazione | 🟡 batch fatto; sincronizzazione da fare |
| T6 | `report`: PDF, CSV, JSON, JSONL | 🟡 PDF, CSV, JSON fatti; mancano JSONL e l'insieme |
| T7 | `ui-egui`: libreria, dettaglio, analisi, impostazioni, raccolte, statistiche | ✅ |
| T8 | Canale di controllo, parità col Python | ⬜ |

## Prossimi passi

1. **Sorgenti/Canale di controllo** (T8): per pilotare l'app in modo deterministico.
2. **Sincronizzazione cartelle** `SYNCED`/`MISSING`/`PARTIAL` (T5).
3. **XMP**: leggere/scrivere rating e keyword.
4. **Recupero JSON** e **punteggi pesati** (T3/T4).

## Registro dei progressi

- **2026-10-13 (funzioni):** analisi con Ollama e ChatGPT; archivio SQLite; statistiche; raccolte per tag;
  valutazione a stelle; esporta PDF; import dal database Python; batch con avanzamento; modulo Setup.
  Funzioni 30% → 55%.
- **2026-10-13:** colori e misure dal tema del riferimento; modulo Statistiche.
- **2026-10-13:** layout sul riferimento LightCraft.
- **2026-10-13:** analisi con Ollama, archivio SQLite, veste grafica.
- **2026-10-13:** prima fetta usabile; scheletro Rust.

## Cronologia delle revisioni

| Data | Modifica | Riassunto |
|---|---|---|
| 2026-10-13 | minor | ChatGPT, statistiche, raccolte, stelle, PDF, import, batch, Setup; 55% |
| 2026-10-13 | minor | Colori e misure dal riferimento |
| 2026-10-13 | minor | Layout sul riferimento |
| 2026-10-13 | minor | Analisi con Ollama, archivio SQLite |
| 2026-10-13 | major | Creata: scheletro Rust, regole, architettura e piano |
