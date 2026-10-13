# Architettura di Postcritica

> **Ultima revisione:** 2026-10-13 · **Ultimo aggiornamento:** 2026-10-13 · **Modifica:** major (creato) · **Obiettivo:** portare il programma Python in un'app desktop Rust

Come è fatto il programma di destinazione e come si arriva dal programma Python esistente. Il piano a
tappe è in [`../plan/execution-plan.md`](../plan/execution-plan.md); le regole di lavoro in
[`../AGENTS.md`](../AGENTS.md).

## 1. Obiettivo e riferimenti

- **Prodotto finale:** app **desktop nativa in Rust** (egui/eframe), offline per default, con archivio
  **SQLite** locale e analisi via **Ollama** (locale) o **ChatGPT/OpenAI**.
- **Riferimento funzionale:** il programma Python in
  `D:\Programmi Sviluppo\Unificato\postcritica-Critic` (è un programma nostro: non c'è clean-room da
  rispettare, solo da **non imitarne** la struttura monolitica).
- **Lingua:** commenti, docstring e stringhe utente in italiano; identificatori in inglese.

## 2. Strati

Le dipendenze puntano solo verso il basso. La tabella è verificata da `cargo xtask layers`.

| Livello | Crate | Dipende da |
|---|---|---|
| L5 | `ui-egui`, `apps/postcritica` | engine, (develop?) |
| L4 | `engine` | analysis, archive, ai, report, metadata |
| L3 | `report` | analysis, archive |
| L2 | `archive`, `ai` | metadata, (raster) |
| L1 | `metadata`, `analysis` | raster, color |
| L0 | `raster`, `color`, `geom` (se servono) | — |

Regola dura: **niente sotto il livello L5 dipende da egui/eframe/winit**. È il confine che tiene l'app
testabile e la logica portabile a una CLI o a un server.

## 3. Le crate

- **`raster` (L0)** — buffer di pixel, conversione a RGBA8, ridimensionamento, codifica/decodifica di
  base. Punto unico per "dammi i pixel di questa immagine".
- **`metadata` (L1)** — lettura e scrittura **EXIF / XMP / IPTC**: i campi che il programma mostra, il
  sidecar XMP, gli XPKeywords/XPComment, il rating. Scritture sempre **atomiche** (temporaneo + rename) e
  mai sul file RAW. Sostituisce la lettura EXIF di `insight_engine.py` e la scrittura di
  `metadata_writer.py`.
- **`analysis` (L1)** — ispezione **locale**, senza rete: profilo visivo (luminosità, contrasto,
  nitidezza, clipping, colorfulness), indicatori tecnici (rischio micromosso dalla regola del reciproco,
  rischio rumore dall'ISO), anteprime/crop per il prompt. Porta `insight_engine.py`.
- **`archive` (L2)** — persistenza **SQLite**: schema, migrazioni idempotenti, query, vocabolario dei tag
  (con sinonimi italiani), raccolte, batch, statistiche, ricerca. Dietro un'interfaccia (repository), così
  l'engine non conosce SQL. Porta `storage.py`.
- **`ai` (L2)** — integrazione modelli: un'interfaccia comune ai **provider** (Ollama, OpenAI), la
  costruzione del **prompt** e degli **schemi JSON**, il recupero di JSON troncato/malformato, il
  dimensionamento del contesto, la gestione delle **chiavi** (mai nei log). Porta la parte AI di `app.py`.
- **`report` (L3)** — generazione **PDF**, **CSV**, **JSON**, **JSONL** dal record dell'analisi. Porta
  `build_pdf`/`export_*` di `app.py`.
- **`engine` (L4)** — **tutti i comandi** e l'orchestrazione: analizza una foto, analizza una cartella,
  batch persistente e riprendibile, sincronizzazione con albero `SYNCED`/`MISSING`/`PARTIAL`, punteggi e
  parsing dei report, azioni su tag/raccolte/archivio. È la logica applicativa che in `app.py` stava
  dentro le rotte. Espone ogni azione con id, parametri, `enabled()`, `run()`.
- **`ui-egui` (L5)** — interfaccia: griglia libreria, dettaglio con istogramma e metadati, pannello di
  analisi, impostazioni, raccolte, statistiche, batch, report. Porta `templates/index.html` e le rotte.

## 4. Mappa dalla struttura Python

| Oggi (Python) | Dove andrà (Rust) |
|---|---|
| Rotte Flask `app.py:2816-4758` | `engine` (azioni) + `ui-egui` (viste) |
| Orchestrazione `_analyze_file_payload`, `_batch_worker`, `_sync_worker`, `_write_record` | `engine` |
| Punteggi e parsing report `calculate_total`, `extract_json`, `recover_critic_report` | `engine` (o `ai` per il parsing) |
| `insight_engine.py` (EXIF + profilo visivo + anteprime) | `metadata` (EXIF/XMP) + `analysis` (profilo/anteprime) |
| `storage.py` (`LocalArchive`, 11 tabelle) | `archive` |
| AI `call_critic`, `call_chatgpt`, prompt, schemi | `ai` |
| PDF `build_pdf`, export CSV/JSON/JSONL | `report` |
| `metadata_writer.py` | `metadata` |
| `ollama_startup.py` (avvio servizio) | `ai` (o `engine`) |
| Costanti, profili, schemi JSON, prompt | i moduli competenti (`ai`, `engine`) |
| `templates/index.html` + rotte | `ui-egui` |

## 5. Modello dati (SQLite)

Dal programma Python, 11 tabelle (`SCHEMA_VERSION = 11`, ma con migrazioni solo fino a v5: nel port si
introduce una catena di migrazioni **coerente e numerata a dovere**).

- `analyses` — il record centrale: `id`, date, sorgente e chiave, nome/percorso file, hash, `shot_at`,
  titolo, genere, profilo, modello, punteggio, durata, intenzione, anteprima, e i blob JSON
  (`metadata_json`, `tags_json`, `insight_json`, `report_json`). Indici su data/percorso/hash/punteggio.
- `search_snippets` + `search_terms` — indice della ricerca semantica locale (faccette: soggetto,
  atmosfera, geometria, colori, luogo, distintivo).
- `external_evaluations` — valutazioni di un secondo provider/modello, con hash della risposta.
- `settings` — chiave/valore (impostazioni, `SCHEMA_VERSION`, chiave API protetta).
- `vocabulary` — vocabolario dei tag (nome unico, categoria).
- `collections` + `collection_items` — raccolte manuali o generate.
- `batch_jobs` + `batch_items` — batch persistente, riprendibile, con stato per item.

Le chiavi e i JSON restano in italiano come oggi. Nel port: migrazioni numerate, `PRAGMA` sensati,
apertura robusta (un DB corrotto è un errore, non un crash) e **niente variabili globali** per lo switch
fra database.

## 6. Integrazione AI

Un solo punto d'ingresso (`engine` chiama `ai`) con due provider dietro la stessa interfaccia:

- **Prompt e schema** (comuni): i profili critici, i profili di produzione, le allerte tecniche, il
  vincolo sui tag (max 40, spazi mai underscore), l'intenzione, e lo **schema JSON** del report.
- **Ollama:** `/api/generate` con immagini (vista completa + crop), `num_ctx` dimensionato per famiglia di
  modello, `temperature 0.2`, `seed 42`; **fallback per memoria** su modelli più leggeri; un solo retry con
  contesto ampliato se il JSON non è recuperabile; metriche salvate nel report.
- **OpenAI:** Chat Completions con Structured Outputs (`strict`), immagini in `image_url` a `detail:high`;
  errori HTTP mappati su errori tipizzati.
- **Chiavi:** protette a riposo (come il DPAPI di oggi) e **mai restituite** in lettura o nei log; in
  alternativa variabile d'ambiente.
- **Recupero JSON** (`recover_critic_report`, `complete_partial_report`, `repair_truncated_json`): porta in
  `ai`, con test propri — è una delle parti più delicate e oggi meno testate.

## 7. Comandi (engine)

Ogni azione è un comando con id stabile e parte del registro (`command_specs()`), così UI, CLI e futuro
canale di controllo la invocano allo stesso modo. Gruppi principali (dal programma Python):

`photo.inspect`, `photo.analyze`, `folder.analyze`, `folder.sync`, `batch.start/pause/resume`,
`archive.get/history/edit/delete/clear`, `tag.*` (add/rename/merge/delete/normalize/rebuild),
`collection.*` (create/update/add/remove/generate/semanticSearch), `metadata.write`, `report.pdf/csv/json`,
`settings.get/set`, `rating.external.*`.

## 8. Regole di dipendenza e come si verificano

- Niente dipendenze che risalgono gli strati (verificato da `cargo xtask layers`).
- Niente egui sotto L5.
- `archive` e `ai` non conoscono l'engine; l'engine non conosce egui.
- I tipi condivisi (un record di analisi, un esito) vivono nella crate più in basso che ne ha bisogno,
  non in un "types" onnicomprensivo.

## 9. Cosa NON portare

- Il **monolite** `app.py`: la logica delle rotte va nei comandi, la presentazione nella UI.
- Le **variabili globali mutabili** (`archive`, `DB_PATH`, `_database_profiles`) e i `threading.Lock`
  globali: lo stato vive dove serve e lo switch di database è un valore, non un globale.
- Gli **`except Exception`** silenziosi (oltre 30 in `app.py`): ogni errore o si gestisce o si propaga.
- Il **`print`** al posto del logging: si usa `log`.
- Il PDF duplicato (`build_pdf` + `_build_pdf_legacy`): una sola implementazione.
- I test legati a stringhe di `index.html`: si testano comportamenti, non markup.
- I **path assoluti** hard-coded in test e script.

## 10. Cronologia delle revisioni

| Data | Modifica | Riassunto |
|---|---|---|
| 2026-10-13 | major | Creata la mappa della migrazione dal programma Python all'app Rust |
