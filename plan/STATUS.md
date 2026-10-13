# Stato

**Tappa corrente:** le funzioni principali sono in piedi (analisi, archivio, statistiche, raccolte, setup, batch, PDF, import).
**Prossimo task:** canale di controllo (T8), sincronizzazione cartelle (T5), XMP.

## Fatto
- Regole e standard in italiano; architettura, piano, documenti; repository su GitHub.
- **Analisi** con Ollama (locale) e ChatGPT (OpenAI): report con punteggio, tag, forze, debolezze.
- **Archivio SQLite** (rusqlite): analisi, impostazioni, valutazione a stelle, import dal database Python.
- **Batch**: «Analizza tutte» con barra di avanzamento.
- **Libreria** (griglia per data, EXIF, profilo visivo), **Statistiche** (4 grafici), **Raccolte per tag**,
  **Setup** (provider/modelli/chiave).
- **Esporta PDF** scritto a mano (niente dipendenze).
- Veste grafica con i **colori e le misure del riferimento LightCraft**.
- **Eseguibile release** in `dist/postcritica.exe`.

## In corso
- Nulla di aperto: l'ultimo blocco è chiuso e committato.

## Blocchi
- Nessuno.

## Assunzioni registrate
- Interfaccia: app desktop nativa egui/eframe, con il tema del riferimento.
- Codice: commenti/stringhe in italiano, identificatori in inglese.
- Provider AI: Ollama e ChatGPT, scelti nel Setup; la chiave OpenAI resta nell'archivio locale.
- SQLite è l'unica dipendenza nativa ammessa.
- Obiettivo di fase: parità funzionale col Python (55% secondo la checklist).

## Metriche
- Test: unitari verdi; test d'integrazione reale con Ollama (`--ignored`).
- Eseguibile release: ~12 MB.

## Decisioni del proprietario in sospeso
- Canale di controllo e MCP (tappa T8).
- Nome delle crate: al momento `postcritica-*`.
