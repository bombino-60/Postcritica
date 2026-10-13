# PhotoInsight Critic — istruzioni per gli agenti

PhotoInsight Critic è un'**applicazione desktop nativa in Rust** per l'analisi tecnica, la catalogazione,
la critica fotografica, l'archivio, le statistiche e le raccolte. Tutti i dati restano in un database
**SQLite** locale; le analisi usano **Ollama** (modelli vision in locale) oppure **ChatGPT Vision**. La UI
è **egui/eframe**, come LightCraft. Il codice ha commenti, docstring e stringhe utente in **italiano**; gli
identificatori sono in inglese.

Questa cartella (`D:\Programmi Sviluppo\Photocritic`) è la casa del progetto. Il **riferimento funzionale**
è il programma Python esistente `D:\Programmi Sviluppo\Unificato\PhotoInsight-Critic`: va portato, non
imitato nella struttura. Il piano della migrazione è in [`plan/execution-plan.md`](plan/execution-plan.md);
l'architettura di destinazione in [`docs/architecture.md`](docs/architecture.md).

> **Questo file è l'unica fonte di verità su come si lavora qui.** Adattato dalle regole di LightCraft e
> dagli standard in [`standards/`](standards/). Tutto ciò che è marcato `TODO(photocritic):` è una
> decisione che solo il proprietario può prendere: lasciala e chiedi una volta sola.

## Inizia ogni sessione da qui
1. Leggi `plan/STATUS.md` (tappa corrente, prossimo task, blocchi) e `plan/execution-plan.md` (le tappe
   della migrazione).
2. Leggi la sezione pertinente di `docs/architecture.md` e il README della crate che tocchi.
3. **Sappi dove siamo:** leggi [`ROADMAP.md`](ROADMAP.md) e, per metodo ed evidenze,
   [`docs/target-app-parity.md`](docs/target-app-parity.md). Una riga della checklist conta una funzione
   che *esiste*, non una che è *buona*.
4. **Scegli il lavoro** da [`docs/gaps.md`](docs/gaps.md), partendo dall'alto. Chi atterra una funzione
   aggiorna la sua riga in [`docs/parity-checklist.md`](docs/parity-checklist.md) e la voce dei gap nello
   **stesso commit**, più la riga *Ultimo aggiornamento* e la cronologia dei documenti (come da
   [`standards/progress-docs.md`](standards/progress-docs.md)).
5. Segui il protocollo di operazione autonoma di `plan/execution-plan.md`.

## Mai in crash (viene prima del lavoro sulle funzioni)
Un file malformato, un input sbagliato, un modello che risponde spazzatura, un database corrotto o il
disco pieno producono un **errore**, mai un panic. **Standard completo:
[`standards/never-crash.md`](standards/never-crash.md).** In sintesi:
- Il codice non di test **non va mai in panic**: niente `unwrap()`, `expect()`, `panic!`, `unreachable!`,
  `todo!`, `unimplemented!`, niente `unsafe`. Errore via `Result` e `?`; una funzione non finita restituisce
  "non supportato" o è disattivata.
- `unsafe` solo in una crate helper isolata; ogni altra radice porta `#![forbid(unsafe_code)]`.
- **Input e file sono ostili:** `get()` invece di `[…]`, aritmetica controllata, allocazioni
  dimensionate dall'input limitate, ricorsione con un tetto, cast validati.
- **Rete e AI:** timeout e retry limitati; una risposta non-JSON si recupera o diventa un errore; le chiavi
  API non finiscono nei log.
- **Scritture atomiche** (file temporaneo + rename); il database corrotto è un errore, non un crash.
- **Ultima rete:** panic hook + `catch_unwind` attorno al dispatch delle azioni; `panic = "unwind"`.
- **Dimostralo:** ogni correzione di crash atterra con un test di regressione che prima andava in panic.

## Non negoziabili
- **Lingua:** commenti, docstring e stringhe utente in **italiano**; identificatori di codice in inglese.
  I termini tecnici e i nomi di librerie restano come sono. **TODO(photocritic):** vale anche per i
  messaggi di errore mostrati all'utente (sì, in italiano).
- **Puro Rust** dove possibile: evita le dipendenze native. **Unica eccezione ammessa: SQLite**, perché
  l'archivio del programma *è* SQLite e i dati esistenti vanno letti (una crate di binding a SQLite, con
  il suo codice C incorporato, è accettata e va documentata qui). Gli equivalenti di `rawpy`/`pyexiv2` si
  cercano fra le crate Rust; dove non esistono, la funzione è "non supportata" finché non c'è una
  soluzione Rust con licenza compatibile (i decoder RAW GPL non si possono usare).
- **Stratificazione.** Le dipendenze puntano in una sola direzione, dalla UI verso il basso:
  `ui-egui` → `engine` → (`analysis`, `archive`, `ai`, `report`, `metadata`). **Niente sotto `ui-egui`
  dipende da egui/eframe/winit.** La mappa e la tabella sono in `docs/architecture.md`; `cargo xtask
  layers` la verifica.
- **Tutto è un comando.** Un'azione ottiene un id, un'etichetta, un percorso di menu, una scorciatoia, dei
  parametri, `enabled()` e `run()`. UI, CLI, canale di controllo e MCP distribuiscono per id. (Come
  LightCraft; l'app Python non aveva questo schema e se ne sente la mancanza.)
- **Test:** `cargo test --workspace` deve passare. Ogni funzione nuova porta almeno un test che fallirebbe
  senza di essa.
- **Gate di qualità** prima di ogni commit: `cargo xtask ci` — cioè `cargo fmt --all --check` ·
  `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test --workspace` · `cargo xtask
  layers` · `cargo xtask assets`. **Non indebolire un gate per far passare un commit.**
- **Commit:** un task per commit (`T2.3: estrae il livello archivio`). Solo stati verdi. Il messaggio
  termina con la riga di attribuzione richiesta dall'ambiente.

## Stato git
Il progetto è un **repository git** (inizializzato in T0), **senza remote**. Committa piccolo e spesso:
un crash deve perdere minuti, non ore.
- **Mai committare**: il database (`*.db`), gli output, i media, i file di build.
- Un task per commit, con l'id del task nel messaggio; termina con la riga di attribuzione dell'ambiente.

## Asset: font, icone, immagini
- Regole complete: [`standards/fonts.md`](standards/fonts.md).
- Solo asset **originali** o con **licenza aperta**, registrati in `assets/ATTRIBUTION.md` con licenza (e
  testo quando richiesto).
- **Non usare** icone, immagini, font o screenshot di altri prodotti, nemmeno come segnaposto.
- Icone come **vettori disegnati in codice** (originali); font per script con catena di fallback.
- I font vivono in `storytold/craft-fonts` come per LightCraft, non in questo repo.

## Corpora e file di test
- Regole complete: [`standards/test-corpora.md`](standards/test-corpora.md).
- I file di test piccoli e con licenza aperta stanno con i test; i corpus grandi si scaricano, fissati e
  verificati con sha256 in una cartella in `.gitignore`. Un corpus assente **salta** il test.
- **Mai committare** il database o i media grandi.

## Eseguire e guardare l'app
- **Avvio:** `cargo run -p photoinsight` (binario desktop). **TODO(photocritic):** aggiungere un canale di
  controllo (JSON-lines su una porta locale, come LightCraft) per pilotare l'app in modo deterministico —
  senza, gli agenti non possono guidarla.
- **Non inviare mai input sintetico a livello di sistema operativo** (tasti o clic via automazione): va
  alla finestra in primo piano. Pilota l'app solo via canale di controllo o snapshot headless.
- **Guarda il risultato:** dopo una modifica visibile, esegui l'app e guarda la schermata; per i test non
  presidiati preferisci uno snapshot headless.
- Ollama e ChatGPT si contattano **solo** quando sono il provider scelto.

## Test e prestazioni
- Test unitari accanto al codice; test di integrazione che guidano l'engine su un archivio temporaneo.
- **Misura, non indovinare:** per le analisi AI registra modello usato, token e durata; per l'ispezione, il
  tempo di lettura EXIF su RAW grandi. Budget di riferimento: aprire l'archivio e mostrare la griglia
  entro ~200 ms su una libreria media.
- Non committare media né database.

## Mappa del codice
La struttura a cui si tende — dettaglio in [`docs/architecture.md`](docs/architecture.md):

```
Cargo.toml                 workspace
crates/
  metadata/                (L1) lettura/scrittura EXIF · XMP · IPTC
  analysis/                (L1) ispezione locale: profilo visivo, indicatori, anteprime
  archive/                 (L2) SQLite: schema, migrazioni, query, ricerca
  ai/                      (L2) provider (Ollama, OpenAI), prompt, schemi, chiavi
  report/                  (L3) PDF · CSV · JSON · JSONL
  engine/                  (L4) comandi e orchestrazione: analisi, batch, sincronizzazione, punteggi
  ui-egui/                 (L5) interfaccia egui, viste, pannelli
apps/
  photoinsight/            binario desktop
xtask/                     tooling: ci, layers, assets, parity
scenes/  (eventuale)       foto dimostrative generate proceduralmente
docs/  standards/  plan/  assets/
```

## Crediti dei contributori (finestra Informazioni)
- **TODO(photocritic):** adotta, adatta o cancella. Se adottato, vale la regola di LightCraft: un file di
  statistiche generato più un file di nomi modificato a mano; l'agente chiede **una volta** al proprio
  umano se vuole essere accreditato con più dello username, e aggiunge **solo la sua** voce.
