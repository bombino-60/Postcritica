# Standard: mai in crash

> **Standard di Postcritica.** Adattato dallo standard condiviso `never-crash` di `craftrules`
> (repository privato), riscritto in italiano e per Rust perché il progetto resti autosufficiente.

Questa regola viene **prima** di ogni funzione. Postcritica legge foto e mantiene un archivio
locale: un crash a metà di un'analisi, di un batch o di una scrittura può perdere lavoro. Un file
malformato (un RAW o un JPEG corrotto, un XMP con struttura assurda, un EXIF incoerente), un input
utente sbagliato, un modello AI che risponde spazzatura, un database corrotto, la rete che cade o il
disco pieno devono produrre un **errore che l'utente o l'agente può capire**, mai un panic.

**Standard completo: craftrules `never-crash` (privato).** In sintesi, applicato qui:

## Il codice non di test non va mai in panic

- Niente `unwrap()`, `expect()`, `panic!`, `unreachable!`, `todo!`, `unimplemented!` e niente `unsafe`.
- Restituisci il tipo di errore della crate tramite `Result` e `?`; usa `ok_or(..)?`, `let … else`,
  `if let` o un ripiego (`unwrap_or…`) solo dove non può corrompere silenziosamente un documento.
- Una funzione non finita **restituisce un errore "non supportato"** oppure è disattivata.
- Unica eccezione: un valore letterale **dimostrabilmente infallibile**, come
  `#[allow(clippy::expect_used)]` + `.expect("perché non può fallire")`.

## `unsafe`

- `unsafe` vive solo in **una singola crate helper isolata e ben testata**, se mai servisse. Ogni altra
  radice di crate di produzione porta `#![forbid(unsafe_code)]`.
- Una nuova esigenza di `unsafe` si mette in una crate di quel tipo: `// SAFETY:` su ogni blocco, un'API
  sicura, un ripiego sicuro dove possibile, e una riga nel README che la nomina.

## I numeri derivati dall'input sono ostili

- Usa `get()` invece di `[i]` / `[a..b]` per offset che vengono da file, utente, agente o aritmetica su
  di essi.
- Aritmetica controllata o saturante per lunghezze, offset e conteggi.
- Niente divisione per zero, NaN/infinito o cast negativi a `usize`.
- Limita le allocazioni dimensionate dall'input (per esempio non riservare memoria per un'immagine che
  *dichiara* di essere enorme); taglia le stringhe solo ai confini di carattere.

## Ricorsione, lock e thread

- **Limita la ricorsione** con limiti di profondità o insiemi di elementi visti (alberi EXIF/IFD, metadati
  annidati, collezioni).
- **Non propagare il panico:** `lock().unwrap_or_else(PoisonError::into_inner)` oppure un errore; i join
  dei thread di lavoro sono `Result`.
- I worker di batch e sincronizzazione **non restano "in corso"** dopo un riavvio: lo stato si riconcilia
  all'apertura dell'archivio (il programma Python ha già questa logica).

## Rete, AI e database

- Le chiamate a Ollama/OpenAI hanno **timeout** e **retry limitati**; una risposta non-JSON si recupera o
  diventa un errore, mai un panic.
- Le **scritture sono atomiche**: file temporaneo + rename; il database si apre con `PRAGMA` sensati e
  l'errore di corruzione diventa un messaggio, non un crash.
- Le **chiavi API** non finiscono mai nei log né nelle risposte d'errore.

## Ultima rete di sicurezza

- Un **panic hook** più `catch_unwind` attorno al dispatch delle azioni e all'import/export trasforma un
  panic sfuggito in una finestra d'errore e conserva l'archivio. È una rete di sicurezza, non una
  licenza: tieni `panic = "unwind"` sul nativo.

## Dimostralo

Ogni correzione di crash atterra con un **test di regressione sintetico** che andava in panic prima della
correzione. Un fix senza il test che lo prova non è completo.

## Applicazione automatica (clippy)

- Un `clippy.toml` alla radice consente `unwrap`/`expect`/`panic`/indicizzazione **solo nei test**.
- Ogni radice di crate di produzione (`lib.rs`, il `main.rs` di ogni binario) porta:

  ```rust
  #![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic,
          clippy::unimplemented, clippy::todo, clippy::unreachable)]
  ```

- **Non** in `[workspace.lints]`: quelli colpirebbero anche i test di integrazione, gli esempi e i
  benchmark. Le crate nuove nascono con l'attributo.
