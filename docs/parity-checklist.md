# Checklist di funzioni di Postcritica

> **Ultima revisione:** 2026-10-13 · **Ultimo aggiornamento:** 2026-10-13 · **Modifica:** minor (canale di controllo, selettore del modello Ollama, UI leggibile, esportazioni, istogramma) · **Obiettivo:** parità funzionale con il programma Python

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
| B. Analisi AI (AI) | 3 | 0 | 2 | 0 | 3/3 (100%) | 0/2 (0%) |
| C. Archivio (ARCHIVE) | 3 | 1 | 3 | 0 | 3/4 (75%) | 0/2 (0%) |
| D. Batch e sincronizzazione (JOB) | 1 | 2 | 3 | 0 | 1/4 (25%) | 0/2 (0%) |
| E. Tag e raccolte (TAXONOMY) | 1 | 1 | 5 | 0 | 1/4 (25%) | 0/3 (0%) |
| F. Esportazione e report (EXPORT) | 2 | 1 | 2 | 0 | 2/3 (67%) | 0/2 (0%) |
| G. Interfaccia (UI) | 9 | 0 | 0 | 0 | 4/4 (100%) | 5/5 (100%) |
| **Totale** | 21 | 7 | 16 | 0 | 15/25 (60%) | 2/18 (11%) |

Completamento pesato (✅ = 1, 🟡 = ½, 🚫 escluso): **56%** di 44 righe in ambito.
<!-- /parity:summary -->

## A. Ispezione tecnica (INSPECT)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-INSPECT-EXIF | Legge EXIF: fotocamera, obiettivo, ISO, f/, tempo, focale, data | P0 | ✅ | `postcritica-metadata` (`read`) | GPS e rating dall'XMP ancora da fare |
| PI-INSPECT-VISUAL | Profilo visivo: luminosità, contrasto, nitidezza, clipping | P0 | 🟡 | `postcritica-analysis` (`VisualProfile::of`) | mancano clipping e colorfulness |
| PI-INSPECT-INDICATORS | Indicatori tecnici: micromosso, rumore | P0 | 🟡 | `postcritica-analysis` (`shake_risk`, `noise_risk`) | calcolati, non ancora mostrati |
| PI-INSPECT-PREVIEW | Anteprima e crop per il prompt | P1 | ✅ | `postcritica-raster` (`decode_thumbnail`, `encode_png`) | il crop per il prompt arriverà |
| PI-INSPECT-RAW | Lettura dei RAW (CR3, NEF, ARW, …) | P1 | ⬜ | | serve una crate Rust (i decoder GPL sono esclusi) |

## B. Analisi AI (AI)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-AI-OLLAMA | Analisi via Ollama (vision) | P0 | ✅ | `postcritica-ai` (`OllamaCritic`) | test `ollama_live` |
| PI-AI-OPENAI | Analisi via ChatGPT/OpenAI | P0 | ✅ | `postcritica-ai` (`OpenAiCritic`) | serve una chiave nel Setup |
| PI-AI-PROMPT | Prompt e schema JSON del report | P0 | ✅ | `postcritica-ai` (`build_prompt`, `report_schema`, `parse_report`) | |
| PI-AI-RECOVER | Recupero di JSON troncato/malformato | P1 | ⬜ | | `parse_report` è tollerante; il recupero vero no |
| PI-AI-TEXT | Modello testuale per le raccolte | P1 | ⬜ | | |

## C. Archivio (ARCHIVE)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-ARCHIVE-SQLITE | Archivio SQLite con schema e migrazioni | P0 | ✅ | `postcritica-archive` (`SqliteArchive`) | |
| PI-ARCHIVE-SAVE | Salva/aggiorna un record, senza duplicati | P0 | ✅ | `SqliteArchive::save` (upsert per fonte) | |
| PI-ARCHIVE-HISTORY | Elenco dell'archivio | P0 | ✅ | modulo `Archivio` | mancano i filtri |
| PI-ARCHIVE-EDIT | Modifica tag/descrizione/rating/GPS | P0 | 🟡 | stelle della valutazione; import dal Python | mancano la modifica di tag e descrizione |
| PI-ARCHIVE-EXTERNAL | Valutazioni esterne (secondo provider) | P1 | ⬜ | | |
| PI-ARCHIVE-BACKUP | Backup del database | P1 | ⬜ | | |
| PI-ARCHIVE-CLEAR | Svuota l'archivio | P2 | ⬜ | | |

## D. Batch e sincronizzazione (JOB)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-JOB-BATCH | Batch: analizza una cartella | P0 | ✅ | «Analizza tutte» | una foto per volta, con avanzamento |
| PI-JOB-RESUME | Riprende un batch interrotto | P0 | 🟡 | la coda salta le foto già analizzate | non sopravvive al riavvio |
| PI-JOB-CORRUPT | Classifica item corrotti senza fermarsi | P0 | 🟡 | un file illeggibile è un errore e il batch continua | |
| PI-JOB-RETRY | Retry limitati sugli errori transitori | P0 | ⬜ | | |
| PI-JOB-SYNC | Sincronizzazione cartelle SYNCED/MISSING/PARTIAL | P1 | ⬜ | | |
| PI-JOB-SCAN | Scansione/audit di una cartella | P1 | ⬜ | | |

## E. Tag e raccolte (TAXONOMY)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-TAX-TAGS | Vocabolario dei tag, con conteggi | P0 | 🟡 | pannello Tag; modulo Raccolte | manca un vocabolario persistente con sinonimi |
| PI-TAX-NORMALIZE | Normalizza/uni/rinomina/elimina tag | P0 | ⬜ | | |
| PI-TAX-DATE | Tag data `gg Mese anno` | P0 | ⬜ | | |
| PI-TAX-COLLECTIONS | Raccolte | P0 | ✅ | modulo Raccolte per tag | solo per tag; le raccolte manuali mancano |
| PI-TAX-IMPORT | Importa raccolte da testo | P1 | ⬜ | | |
| PI-TAX-GENERATE | Genera raccolte via modello testuale | P1 | ⬜ | | |
| PI-TAX-SEMANTIC | Ricerca semantica locale | P1 | ⬜ | | |

## F. Esportazione e report (EXPORT)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-EXPORT-CSV | Esportazione CSV (separatore `;`, BOM) | P0 | ✅ | `postcritica-report::to_csv` | |
| PI-EXPORT-PDF | Report PDF | P0 | ✅ | `postcritica-report::report_pdf` | una pagina, senza immagini |
| PI-EXPORT-JSON | Esportazione JSON | P0 | 🟡 | `postcritica-report::record_to_json` | record singolo; manca l'insieme |
| PI-EXPORT-JSONL | Esportazione JSONL per il fine-tuning | P1 | ⬜ | | |
| PI-EXPORT-METADATA | Scrive XMP/JPEG (copia, mai il RAW) | P1 | ⬜ | `postcritica-metadata` | scrittura da fare |

## G. Interfaccia (UI)

| Id | Funzione | Livello | Stato | Evidenza | Manca / note |
|---|---|---|---|---|---|
| PI-UI-SHELL | Finestra con la veste grafica del riferimento | P0 | ✅ | `postcritica-ui-egui` (`theme`) | palette e misure copiate da LightCraft |
| PI-UI-LIBRARY | Griglia raggruppata per data | P0 | ✅ | modulo Libreria | |
| PI-UI-DETAIL | Dettaglio con metadati, profilo visivo e report | P0 | ✅ | pannello Analisi | manca l'istogramma |
| PI-UI-ANALYSIS | Pannello del report | P0 | ✅ | pannello Analisi | |
| PI-UI-SETTINGS | Setup: provider, modelli, chiave | P1 | ✅ | modulo Setup | |
| PI-UI-COLLECTIONS | Raccolte | P1 | ✅ | modulo Raccolte | |
| PI-UI-STATS | Statistiche con grafici | P1 | ✅ | modulo Statistiche | quattro grafici |
| PI-UI-BATCH | Batch con avanzamento | P1 | ✅ | barra in basso | |
| PI-UI-CONTROL | Canale di controllo locale (JSON-lines su TCP) | P1 | ✅ | `control.rs`, porta 7010 | `status`, `openFolder`, `analyzeAll`, `select`, `setModule`, `quit` |
