# Checklist di funzioni di Postcritica

> **Ultima revisione:** 2026-10-13 · **Ultimo aggiornamento:** 2026-10-13 · **Modifica:** minor (prima fetta usabile: griglia, ispettore EXIF, demo) · **Obiettivo:** parità funzionale con il programma Python

Una riga per funzione del programma, con lo stato **nel port Rust**. La colonna *Evidenza* cita la crate o
il comando quando la funzione c'è. Chi atterra una funzione aggiorna la sua riga nello stesso commit.

- **Id stabili:** `PI-<AREA>-<SLUG>`; mai rinumerare.
- **Livello:** `P0` nucleo · `P1` importante · `P2` più tardi · `OOS` fuori ambito.
- **Stato:** `✅` fatto · `🟡` parziale · `⬜` mancante · `🚫` fuori ambito.

## Riassunto

<!-- parity:summary -->
| Sezione | ✅ | 🟡 | ⬜ | 🚫 | P0 fatti | P1 fatti |
|---|---:|---:|---:|---:|---:|---:|
| A. Ispezione tecnica (INSPECT) | 2 | 2 | 1 | 0 | 1/3 (33%) | 1/2 (50%) |
| B. Analisi AI (AI) | 0 | 0 | 5 | 0 | 0/3 (0%) | 0/2 (0%) |
| C. Archivio (ARCHIVE) | 0 | 1 | 6 | 0 | 0/4 (0%) | 0/2 (0%) |
| D. Batch e sincronizzazione (JOB) | 0 | 0 | 6 | 0 | 0/4 (0%) | 0/2 (0%) |
| E. Tag e raccolte (TAXONOMY) | 0 | 0 | 7 | 0 | 0/4 (0%) | 0/3 (0%) |
| F. Esportazione e report (EXPORT) | 1 | 1 | 3 | 0 | 1/3 (33%) | 0/2 (0%) |
| G. Interfaccia (UI) | 2 | 1 | 5 | 0 | 2/4 (50%) | 0/4 (0%) |
| **Totale** | 5 | 5 | 33 | 0 | 4/25 (16%) | 1/17 (6%) |

Completamento pesato (✅ = 1, 🟡 = ½, 🚫 escluso): **17%** di 43 righe in ambito.
<!-- /parity:summary -->

## A. Ispezione tecnica (INSPECT)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-INSPECT-EXIF | Legge EXIF: fotocamera, obiettivo, ISO, f/, tempo, focale | P0 | ✅ | `postcritica-metadata` (`read`) | GPS e rating (XMP) ancora da fare |
| PI-INSPECT-VISUAL | Profilo visivo: luminosità, contrasto, nitidezza, clipping | P0 | 🟡 | `postcritica-analysis` (`VisualProfile::of`) | mancano clipping e colorfulness |
| PI-INSPECT-INDICATORS | Indicatori tecnici: micromosso, rumore | P0 | 🟡 | `postcritica-analysis` (`shake_risk`, `noise_risk`) | presenti, non ancora mostrati nella UI |
| PI-INSPECT-PREVIEW | Anteprima e crop per il prompt | P1 | ✅ | `postcritica-raster` (`decode_thumbnail`) | i crop per il prompt arrivano con l'analisi AI |
| PI-INSPECT-RAW | Lettura dei RAW (CR3, NEF, ARW, …) | P1 | ⬜ | | serve una crate Rust (i decoder GPL sono esclusi) |

## B. Analisi AI (AI)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-AI-OLLAMA | Analisi via Ollama (vision) | P0 | ⬜ | `postcritica-ai` (interfaccia) | manca la chiamata di rete (T3) |
| PI-AI-OPENAI | Analisi via ChatGPT/OpenAI | P0 | ⬜ | | T3 |
| PI-AI-PROMPT | Prompt e schema JSON del report | P0 | ⬜ | | da portare |
| PI-AI-RECOVER | Recupero di JSON troncato/malformato | P1 | ⬜ | | parte delicata, con test propri |
| PI-AI-TEXT | Modello testuale per le raccolte | P1 | ⬜ | | |

## C. Archivio (ARCHIVE)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-ARCHIVE-SQLITE | Archivio SQLite con schema e migrazioni | P0 | ⬜ | `postcritica-archive` (interfaccia) | c'è solo `MemoryArchive` (T1) |
| PI-ARCHIVE-SAVE | Salva/aggiorna un record, senza duplicati | P0 | 🟡 | `MemoryArchive::save` | dedup per fonte; SQLite in T1 |
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
| PI-EXPORT-CSV | Esportazione CSV (separatore `;`, BOM) | P0 | ✅ | `postcritica-report::to_csv` | |
| PI-EXPORT-JSON | Esportazione JSON | P0 | 🟡 | `postcritica-report::record_to_json` | record singolo; manca l'insieme |
| PI-EXPORT-PDF | Report PDF | P0 | ⬜ | | T6 |
| PI-EXPORT-JSONL | Esportazione JSONL per il fine-tuning | P1 | ⬜ | | |
| PI-EXPORT-METADATA | Scrive XMP/JPEG (copia, mai il RAW) | P1 | ⬜ | `postcritica-metadata` | scrittura in T2 |

## G. Interfaccia (UI)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-UI-SHELL | Finestra dell'app | P0 | ✅ | `postcritica-ui-egui::run` | |
| PI-UI-LIBRARY | Griglia della libreria | P0 | ✅ | `postcritica-ui-egui` (griglia di miniature) | dalla cartella scelta o dalla demo |
| PI-UI-DETAIL | Dettaglio con metadati e profilo visivo | P0 | 🟡 | `postcritica-ui-egui` (ispettore) | manca l'istogramma |
| PI-UI-ANALYSIS | Pannello di analisi col report | P0 | ⬜ | | T7 |
| PI-UI-SETTINGS | Impostazioni (provider, chiave, salvataggio) | P1 | ⬜ | | |
| PI-UI-COLLECTIONS | Raccolte | P1 | ⬜ | | |
| PI-UI-STATS | Statistiche con filtri incrociati | P1 | ⬜ | | |
| PI-UI-BATCH | Batch (avanzamento, pausa, riprendi) | P1 | ⬜ | | |
