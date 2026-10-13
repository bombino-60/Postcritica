# Checklist di funzioni di PhotoInsight Critic

> **Ultima revisione:** 2026-10-13 · **Ultimo aggiornamento:** 2026-10-13 · **Modifica:** major (creata) · **Obiettivo:** parità funzionale con il programma Python

Una riga per funzione del programma, con lo stato **nel port Rust**. La colonna *Evidenza* cita la crate o
il comando quando la funzione c'è. Chi atterra una funzione aggiorna la sua riga nello stesso commit.

- **Id stabili:** `PI-<AREA>-<SLUG>`; mai rinumerare.
- **Livello:** `P0` nucleo · `P1` importante · `P2` più tardi · `OOS` fuori ambito.
- **Stato:** `✅` fatto · `🟡` parziale · `⬜` mancante · `🚫` fuori ambito.

## Riassunto

<!-- parity:summary -->
| Sezione | ✅ | 🟡 | ⬜ | 🚫 | P0 fatti | P1 fatti |
|---|---:|---:|---:|---:|---:|---:|
| A. Ispezione tecnica (INSPECT) | 0 | 1 | 4 | 0 | 0/3 (0%) | 0/2 (0%) |
| B. Analisi AI (AI) | 0 | 0 | 5 | 0 | 0/3 (0%) | 0/2 (0%) |
| C. Archivio (ARCHIVE) | 0 | 1 | 6 | 0 | 0/4 (0%) | 0/2 (0%) |
| D. Batch e sincronizzazione (JOB) | 0 | 0 | 6 | 0 | 0/4 (0%) | 0/2 (0%) |
| E. Tag e raccolte (TAXONOMY) | 0 | 0 | 7 | 0 | 0/4 (0%) | 0/3 (0%) |
| F. Esportazione e report (EXPORT) | 1 | 1 | 3 | 0 | 1/3 (33%) | 0/2 (0%) |
| G. Interfaccia (UI) | 0 | 1 | 7 | 0 | 0/4 (0%) | 0/4 (0%) |
| **Totale** | 1 | 4 | 38 | 0 | 1/25 (4%) | 0/17 (0%) |

Completamento pesato (✅ = 1, 🟡 = ½, 🚫 escluso): **7%** di 43 righe in ambito.
<!-- /parity:summary -->

## A. Ispezione tecnica (INSPECT)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-INSPECT-EXIF | Legge EXIF/XMP: fotocamera, obiettivo, ISO, f/, tempo, focale, GPS, rating | P0 | ⬜ | | `metadata::read` risponde "non supportato" (T2) |
| PI-INSPECT-VISUAL | Profilo visivo: luminosità, contrasto, nitidezza, clipping | P0 | 🟡 | `photoinsight-analysis` (`VisualProfile::of`) | mancano clipping e colorfulness |
| PI-INSPECT-INDICATORS | Indicatori tecnici: micromosso, rumore | P0 | ⬜ | `photoinsight-analysis` (`shake_risk`, `noise_risk`) | presenti ma non collegati alla UI |
| PI-INSPECT-PREVIEW | Anteprima e crop per il prompt | P1 | ⬜ | | decodifica e crop in T2 |
| PI-INSPECT-RAW | Lettura dei RAW (CR3, NEF, ARW, …) | P1 | ⬜ | | serve una crate Rust (i decoder GPL sono esclusi) |

## B. Analisi AI (AI)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-AI-OLLAMA | Analisi via Ollama (vision) | P0 | ⬜ | `photoinsight-ai` (interfaccia) | manca la chiamata di rete (T3) |
| PI-AI-OPENAI | Analisi via ChatGPT/OpenAI | P0 | ⬜ | | T3 |
| PI-AI-PROMPT | Prompt e schema JSON del report | P0 | ⬜ | | da portare |
| PI-AI-RECOVER | Recupero di JSON troncato/malformato | P1 | ⬜ | | parte delicata, con test propri |
| PI-AI-TEXT | Modello testuale per le raccolte | P1 | ⬜ | | |

## C. Archivio (ARCHIVE)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-ARCHIVE-SQLITE | Archivio SQLite con schema e migrazioni | P0 | ⬜ | `photoinsight-archive` (interfaccia) | c'è solo `MemoryArchive` (T1) |
| PI-ARCHIVE-SAVE | Salva/aggiorna un record, senza duplicati | P0 | 🟡 | `MemoryArchive::save` | dedup per fonte fatta; SQLite in T1 |
| PI-ARCHIVE-HISTORY | Elenco dell'archivio, filtri | P0 | ⬜ | | |
| PI-ARCHIVE-EDIT | Modifica tag/descrizione/rating/GPS | P0 | ⬜ | | |
| PI-ARCHIVE-EXTERNAL | Valutazioni esterne (secondo provider) | P1 | ⬜ | | |
| PI-ARCHIVE-BACKUP | Backup del database | P1 | ⬜ | | |
| PI-ARCHIVE-CLEAR | Svuota l'archivio | P2 | ⬜ | | |

## D. Batch e sincronizzazione (JOB)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-JOB-BATCH | Batch persistente e riprendibile | P0 | ⬜ | | T5 |
| PI-JOB-RESUME | Riprende un batch interrotto | P0 | ⬜ | | |
| PI-JOB-CORRUPT | Classifica item corrotti senza fermarsi | P0 | ⬜ | | |
| PI-JOB-RETRY | Retry limitati sugli errori transitori | P0 | ⬜ | | |
| PI-JOB-SYNC | Sincronizzazione cartelle con albero SYNCED/MISSING/PARTIAL | P1 | ⬜ | | |
| PI-JOB-SCAN | Scansione/audit di una cartella | P1 | ⬜ | | |

## E. Tag e raccolte (TAXONOMY)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-TAX-TAGS | Vocabolario dei tag, con sinonimi | P0 | ⬜ | | T1 |
| PI-TAX-NORMALIZE | Normalizza/uni/rinomina/elimina tag | P0 | ⬜ | | |
| PI-TAX-DATE | Tag data `gg Mese anno` | P0 | ⬜ | | |
| PI-TAX-COLLECTIONS | Raccolte manuali | P0 | ⬜ | | |
| PI-TAX-IMPORT | Importa raccolte da testo | P1 | ⬜ | | |
| PI-TAX-GENERATE | Genera raccolte via modello testuale | P1 | ⬜ | | |
| PI-TAX-SEMANTIC | Ricerca semantica locale | P1 | ⬜ | | |

## F. Esportazione e report (EXPORT)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-EXPORT-CSV | Esportazione CSV (separatore `;`, BOM) | P0 | ✅ | `photoinsight-report::to_csv` | |
| PI-EXPORT-JSON | Esportazione JSON | P0 | 🟡 | `photoinsight-report::record_to_json` | c'è il record singolo; manca l'insieme |
| PI-EXPORT-PDF | Report PDF | P0 | ⬜ | | T6 |
| PI-EXPORT-JSONL | Esportazione JSONL per il fine-tuning | P1 | ⬜ | | |
| PI-EXPORT-METADATA | Scrive XMP/JPEG (copia, mai il RAW) | P1 | ⬜ | `photoinsight-metadata` | scrittura in T2 |

## G. Interfaccia (UI)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-UI-SHELL | Finestra dell'app | P0 | 🟡 | `photoinsight-ui-egui::run` | c'è solo la finestra vuota |
| PI-UI-LIBRARY | Griglia della libreria | P0 | ⬜ | | T7 |
| PI-UI-DETAIL | Dettaglio con istogramma e metadati | P0 | ⬜ | | |
| PI-UI-ANALYSIS | Pannello di analisi col report | P0 | ⬜ | | |
| PI-UI-SETTINGS | Impostazioni (provider, chiave, salvataggio) | P1 | ⬜ | | |
| PI-UI-COLLECTIONS | Raccolte | P1 | ⬜ | | |
| PI-UI-STATS | Statistiche con filtri incrociati | P1 | ⬜ | | |
| PI-UI-BATCH | Batch (avanzamento, pausa, riprendi) | P1 | ⬜ | | |
