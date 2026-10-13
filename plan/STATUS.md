# Stato

**Tappa corrente:** il cuore del programma è in piedi — analisi con Ollama, archivio SQLite, veste grafica.
**Prossimo task:** la schermata **Archivio e cronologia** (vedere le analisi salvate), poi il **batch**.

## Fatto
- Regole e standard in italiano; architettura, piano e documenti di avanzamento.
- Workspace Cargo con 8 crate stratificati (L0–L5) + il binario `postcritica`; `xtask` con i gate.
- Licenza MIT OR Apache-2.0; repository collegato a `origin` (`bombino-60/Postcritica`).
- **Analisi di una foto con Ollama** (`gemma3:4b` predefinito, modificabile dalla barra): il report
  (punteggio, tag, punti di forza e debolezza) compare a destra ed è **salvato in SQLite**.
- **Libreria**: griglia di miniature (da una cartella o dalla demo), ispettore con **metadati EXIF** e
  **profilo visivo**.
- **Veste grafica della tua app**: palette scura con i tuoi colori.
- `raster` (decodifica/miniatura/PNG), `metadata` (EXIF), `ai` (Ollama + report), `archive` (SQLite),
  `engine` (ispezione e analisi), `report` (CSV/JSON) — tutte con test.
- **Eseguibile release** in `dist/postcritica.exe`.

## In corso
- Nulla di aperto: la fetta è chiusa e committata.

## Blocchi
- Nessuno.

## Assunzioni registrate
- Interfaccia: app desktop nativa egui/eframe, con la palette della tua app.
- Codice: commenti/stringhe in italiano, identificatori in inglese.
- Provider AI: **Ollama** collegato (modello predefinito `gemma3:4b`); ChatGPT/OpenAI dietro la stessa
  interfaccia, più avanti.
- SQLite è l'unica dipendenza nativa ammessa.
- Obiettivo di fase: **parità funzionale** col Python.

## Metriche
- Test: unitari verdi; test d'integrazione reale con Ollama (`--ignored`).
- Eseguibile release: ~12 MB.

## Decisioni del proprietario in sospeso
- Se introdurre il canale di controllo e MCP (tappa T8).
- Nome delle crate: al momento `postcritica-*`.
