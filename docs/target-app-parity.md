# Parità funzionale: Postcritica (Rust) vs il programma Python

> **Ultima revisione:** 2026-10-13 · **Ultimo aggiornamento:** 2026-10-13 · **Modifica:** major (creato) · **Obiettivo:** parità funzionale con `postcritica-Critic` (Python)

La valutazione di quanto il port Rust è vicino al programma Python. Una riga per funzione è in
[parity-checklist.md](parity-checklist.md); il lavoro ordinato in [gaps.md](gaps.md). Il metodo segue
[`../standards/progress-docs.md`](../standards/progress-docs.md).

## In sintesi

| Numero | Valore | Tipo |
|---|---:|---|
| **Funzioni portate**, checklist | **7%** di 43 righe | **misurato**: ✅ 1, 🟡 4 (peso ½) su 43 |
| **Tappe completate** | **0 di 9** (T0 in corso) | misurato |
| **Pronto per il lavoro reale** | **~0%** | stimato |
| **Utente medio** | **~0%** | stimato |
| **Utente essenziale** | **~0%** | stimato |
| Mancante per **alpha** | **~270–500 h** | stimato; i gap `B` di [gaps.md](gaps.md) |
| Mancante per la **parità** col Python | **~400–800 h** | stimato |
| **Stadio** | **pre-alpha** | vedi [Stadio](#stadio) |

## L'obiettivo e come è misurato

- **Obiettivo:** il **programma Python** esistente, funzione per funzione. `postcritica-Critic` è un
  programma *nostro*: non c'è clean-room, solo la sua mappa funzionale
  (`D:\Programmi Sviluppo\Unificato\postcritica-Critic`, 54 rotte, 11 tabelle, ~7600 righe).
- **Lato Rust:** il workspace di questa cartella, con gli 8 crate stratificati.
- **Lato Python, per riferimento:** le funzioni di `app.py`, `storage.py`, `insight_engine.py`,
  `metadata_writer.py` e i loro test. Non si copia il codice; si portano i comportamenti.
- La verifica di parità vera (una passata end-to-end affiancata) è la tappa T8.

**Presenza non è parità.** Un ✅ qui dice che una funzione è portata e testata, non che si comporta ancora
identica al Python in ogni caso: quello lo si scopre a T8.

## Per dimensione

| Dimensione | Ampienza | Pronto | Mancante | Documento |
|---|---:|---:|---:|---|
| Funzioni (aree sotto) | ~7% | ~0% | 400–800 h | [Funzioni](#funzioni) |
| Ispezione tecnica (EXIF, profilo visivo) | ~15% | ~10% | 60–120 h | [architecture](architecture.md#2-strati) |
| Analisi AI (Ollama, OpenAI) | ~5% | ~0% | 60–110 h | [architecture](architecture.md#6-integrazione-ai) |
| Archivio e dati | ~10% | ~5% | 60–100 h | [architecture](architecture.md#5-modello-dati-sqlite) |
| Interfaccia | ~5% | ~0% | 80–150 h | [architecture](architecture.md#3-le-crate) |
| Qualità (test, errori, gate) | ~30% | ~30% | 20–40 h | [standards](../standards/) |

## Funzioni

I pesi seguono dove il programma Python spende più codice e dove l'utente spende più tempo. Ampiezza e
prontezza sono stimate dalla checklist.

| Area | Peso | Ampiezza | Pronto | Mancante |
|---|---:|---:|---:|---:|
| Ispezione tecnica | 20% | ~15% | ~10% | 60–120 h |
| Analisi AI | 20% | ~5% | ~0% | 60–110 h |
| Archivio e dati | 20% | ~10% | ~5% | 60–100 h |
| Batch e sincronizzazione | 10% | 0% | 0% | 40–70 h |
| Tag e raccolte | 10% | 0% | 0% | 30–60 h |
| Esportazione e report | 10% | ~25% | ~20% | 25–50 h |
| Interfaccia | 10% | ~5% | ~0% | 80–150 h |
| **Pesato** | 100% | **~8%** | **~4%** | **400–800 h** |

## Pronto per il lavoro reale, utente medio ed utente essenziale

Tre numeri (vedi lo [standard dei documenti di avanzamento](../standards/progress-docs.md)). Allo
scheletro sono tutti vicini a zero, perché nessun flusso end-to-end è ancora completo.

| Pubblico | Pronto | Ore per ~95% |
|---|---:|---:|
| Obiettivo pieno (parità col Python) | **~0%** | **400–800 h** |
| Utente medio (analizza, archivia, esporta) | **~0%** | **220–420 h** |
| Utente essenziale (analizza una foto e la salva) | **~0%** | **90–170 h** |

## Come sono calibrate le ore

Le ore sono ore di orologio di un agente. Qui la calibrazione è più facile che in un progetto nuovo: il
programma Python **esiste**, quindi un port è "riprodurre un comportamento noto", non "progettarlo". Le
stime sopra derivano dalla dimensione dei moduli Python (righe per modulo) e dalla difficoltà del Rust
(per esempio l'assenza di un decoder RAW Rust con licenza compatibile è una vera incognita). A fine di
ogni tappa si corregge la stima confrontandola con il tempo davvero impiegato.

## Stadio

- **alpha** — i flussi principali funzionano end to end: **ispeziona una foto, analizza (AI), salva
  nell'archivio, riapri l'archivio, esporta il report**. Oggi non passa.
- **beta** — ampiezza vicina alla parità e nessun gap `B` aperto.
- **1.0** — parità più stabilità e gestione completa degli errori.

**Perché pre-alpha:** il workspace compila e i gate sono verdi, ma nessun flusso end-to-end è completo.

## Cronologia delle revisioni

| Data | Modifica | Riassunto |
|---|---|---|
| 2026-10-13 | major | Creato: scheletro (7% delle funzioni), stadio pre-alpha |
