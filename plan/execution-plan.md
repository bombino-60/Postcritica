# Piano di esecuzione — migrazione a Rust

> **Ultima revisione:** 2026-10-13 · **Ultimo aggiornamento:** 2026-10-13 · **Modifica:** major (creato)

Le tappe per portare PhotoInsight Critic dal programma Python all'app desktop Rust. Ogni tappa produce
qualcosa di **verificabile** e si chiude con i gate verdi. Il lavoro si sceglie da
[`../docs/gaps.md`](../docs/gaps.md); lo stato corrente è in [`STATUS.md`](STATUS.md); l'architettura in
[`../docs/architecture.md`](../docs/architecture.md).

## Regole generali

- Una tappa per volta. Dentro una tappa, un task per commit (`T2.3: …`).
- **Solo stati verdi:** `cargo xtask ci` (fmt, clippy `-D warnings`, test, layers, assets) prima di ogni
  commit.
- Ogni crate nuova nasce con `#![forbid(unsafe_code)]` e gli attributi `#![deny(…)]` di
  [`../standards/never-crash.md`](../standards/never-crash.md).
- Il **riferimento** è sempre il programma Python: quando un comportamento non è chiaro, si legge lì e si
  scrive qui un test che lo fissa.

## T0 — Fondamenta (in corso)

Obiettivo: un workspace che compila vuoto, con i gate attivi.

- `git init`, `README.md`, `.gitignore` (database, output, media, `target/`).
- `Cargo.toml` del workspace; `rustfmt.toml`, `clippy.toml` (unwrap/expect/panic solo nei test).
- `xtask` con `ci`, `layers`, `assets`, `parity`.
- Struttura `docs/`, `standards/`, `plan/`, `assets/`.
- **Gate di tappa:** `cargo xtask ci` verde su un workspace con una crate `version` di prova.

## T1 — `archive` (archivio SQLite)

Obiettivo: il modello dati dietro un'interfaccia, con migrazioni numerate.

- Schema delle 11 tabelle di `storage.py`, con migrazioni idempotenti e **numerate in modo coerente**
  (l'originale si fermava a v5 con `SCHEMA_VERSION = 11`).
- Un tipo `Archive` (repository) con: `save`, `get`, `history`, `update_*`, `delete`, `clear`, statistiche.
- Vocabolario dei tag con i sinonimi italiani; `match_existing_tag`; tag data `gg Mese anno`.
- Apertura robusta: un DB corrotto è un errore tipizzato, non un panic.
- **Strumento di import** dal database del programma Python: uno schema **nuovo** più un comando che
  legge le 11 tabelle del vecchio schema e le traduce, con un report di cosa è stato importato e cosa no.
  Test con un database sintetico piccolo.
- **Test:** round-trip, dedup per percorso e per hash, migrazioni da uno schema vecchio, clear che
  preserva le impostazioni, import dal vecchio schema.

## T2 — `metadata` + `analysis`

Obiettivo: leggere una foto e dire cosa contiene, tutto in locale.

- `metadata`: EXIF/XMP/IPTC in lettura (fotocamera, seriali, obiettivo, ISO, f/, tempo, focale, GPS,
  rating, keyword) e in scrittura (sidecar XMP, XPKeywords/XPComment, rating) — **atomico**, mai sul RAW.
- `analysis`: profilo visivo (luminosità, contrasto, nitidezza, clipping, colorfulness), indicatori
  (micromosso, rumore), anteprime e crop per il prompt, hash sha256.
- Il prompt **non** contiene hash né GPS preciso: solo etichette italiane ammesse (come oggi).
- Formati RAW: si parte da JPEG/PNG/TIFF; i RAW arrivano quando c'è una crate Rust adatta (altrimenti
  "non supportato").
- **Test:** EXIF sintetici, XMP scritto e riletto, un JPEG invalido è un errore, crop sovrapposti.

## T3 — `ai`

Obiettivo: il ponte verso i modelli, con i suoi errori.

- Interfaccia `Provider` con `ollama` e `openai`.
- Prompt (profili critici, allerte tecniche, intenzione, tag max 40) e schema JSON del report.
- Recupero JSON (`recover`, `complete_partial`, `repair_truncated`) con test propri.
- Gestione modelli/vision, fallback per memoria, contesto dimensionato, un retry con contesto ampliato.
- Chiavi protette a riposo e **mai** nei log; variabile d'ambiente come alternativa.
- **Test:** risposte JSON troncate/malformate recuperate o errore tipizzato; timeout; chiave assente.

## T4 — `engine`: analisi singola

Obiettivo: il primo flusso end-to-end, senza UI.

- Comandi `photo.inspect`, `photo.analyze`, `archive.*`, `tag.*`.
- Punteggi e parsing dei report (porta `calculate_total` e dintorni) con test che fissano i valori.
- `metadata.write` che collega un record ai metadati scritti.
- **Gate di tappa:** una foto di test viene ispezionata, analizzata (provider finto nei test), salvata in
  archivio e riletta.

## T5 — `engine`: batch e sincronizzazione

- Batch persistente e riprendibile, con stato per item (`succeeded/skipped/failed/corrupt`) e retry
  limitati; un batch interrotto non resta "in corso".
- Sincronizzazione cartelle con albero `SYNCED`/`MISSING`/`PARTIAL`.
- **Test:** resume dopo interruzione, item corrotto, retry su errore transitorio.

## T6 — `report`

- PDF (una sola implementazione, niente `_build_pdf_legacy`), CSV (separatore `;`, BOM), JSON, JSONL.

## T7 — `ui-egui`

- Griglia libreria, dettaglio con istogramma e metadati, pannello di analisi, impostazioni, raccolte,
  statistiche, batch, report.
- Ogni azione della UI invoca un comando dell'engine.

## T8 — Canale di controllo e parità

- Canale di controllo (JSON-lines) per pilotare l'app come LightCraft; facoltativo MCP.
- Passata di parità contro il programma Python: la checklist diventa la verifica.

## Protocollo di operazione autonoma

Se non indicato diversamente:
- Scegli il prossimo task da `plan/STATUS.md`; se non c'è, dal prossimo task non spuntato in questo piano.
- Non fermarti a chiedere se la decisione non è fra quelle del proprietario; registra le assunzioni in
  `STATUS.md`.
- Le decisioni del proprietario sono: nome e marchio, provider AI e dove stanno le chiavi, la licenza del
  progetto, se introdurre il canale di controllo e MCP, e ogni cambio di scope.
