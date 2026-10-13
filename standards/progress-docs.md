# Standard: documenti di avanzamento

> **Standard di Postcritica.** Adattato dallo standard condiviso `progress-docs` di `craftrules`
> (repository privato), riscritto in italiano perché il progetto resti autosufficiente.

Definisce la forma e le regole dei quattro documenti che dicono **a che punto è Postcritica**,
così che un lettore (umano o agente) possa fidarsi dei numeri senza rifarli.

## I quattro documenti e chi fa cosa

| Documento | Posizione | Risponde a |
|---|---|---|
| `ROADMAP.md` | radice | Dove siamo, in una pagina: stadio, numeri di testa, prossimi passi. |
| `docs/parity-checklist.md` | `docs/` | *La funzione esiste?* Una riga per funzione, schermata e comando. |
| `docs/target-app-parity.md` | `docs/` | *Quanto è buona?* Valutazione con metodo, pesi ed evidenze. |
| `docs/gaps.md` | `docs/` | *Cosa manca e in che ordine?* Elenco ordinato per impatto sull'utente. |

Il principio che li tiene insieme: **presenza ≠ qualità**. Un ✅ nella checklist dice che una funzione
esiste, non che è buona. La qualità si misura in `target-app-parity.md`.

## Intestazione obbligatoria

Ogni documento di avanzamento comincia con una riga quotata, sempre nella stessa forma:

```
> **Ultima revisione:** AAAA-MM-GG · **Ultimo aggiornamento:** AAAA-MM-GG · **Modifica:** major|minor (cosa) · **Obiettivo:** il traguardo
```

- **Ultima revisione** cambia quando si ri-verifica l'insieme; **Ultimo aggiornamento** quando si tocca
  un numero o una riga. Aggiornali nello stesso commit della modifica.
- **Modifica** è `major` per un cambio di metodo o una ri-misura, `minor` per il resto, con una parentesi
  che dice cosa è cambiato.

## Numeri: misurato o stimato, mai ambiguo

- Ogni numero porta un'etichetta: **misurato** (conta qualcosa che esiste: righe di checklist, test,
  file) oppure **stimato** (un giudizio con pesi o sconti scritti).
- I numeri **stimati** dichiarano il metodo. Se un numero è un'ipotesi, scrivilo.
- **Nessuna percentuale sale senza evidenza.** Se non hai nuove prove, il numero non cambia.
- Le **ore** sono ore di orologio di un agente, calibrate sulla storia del progetto, con un intervallo
  quando c'è incertezza.

I numeri di testa di `ROADMAP.md` e `target-app-parity.md` devono coincidere.

## Prontezza: tre numeri

Ogni valutazione porta tre numeri, dal più largo al più stretto (essenziale ⊂ medio ⊂ pieno):

1. **Pronto per il lavoro reale** (obiettivo pieno): somma pesata sulle dimensioni.
2. **Utente medio**: le aree che tocca un utente abituale, per la loro profondità, per sconti
   moltiplicativi scritti.
3. **Utente essenziale**: le funzioni di base ai valori predefiniti, per uno sconto scritto.

Ognuno con le sue ore fino a ~95 %.

## Checklist: una riga per funzione

- **Id stabili** con un prefisso per sezione (per esempio `PI-<AREA>-<SLUG>`, `SCR-…`, `CMD-…`). **Mai
  rinumerare.**
- **Livello:** `P0` nucleo · `P1` importante · `P2` più tardi / di nicchia · `OOS` fuori ambito.
- **Stato:** `✅` fatto · `🟡` parziale (la nota dice cosa manca, o non è verificato) · `⬜` mancante ·
  `🚫` fuori ambito.
- **Evidenza:** cita le funzioni/rotte e i percorsi dei sorgenti. Nessun testo o asset di terzi.
- **Chi atterra una funzione aggiorna la sua riga nello stesso commit.**
- Il blocco riassuntivo è generato dal tooling (`cargo xtask parity --write`); il controllo (parte di
  `cargo xtask ci`) fallisce se un id, un comando o un percorso citato non esiste più.

## Gap: ordinati per impatto

Una voce per mancanza, con: rango, `B` se blocca la prossima fase, il gap in una frase, l'**evidenza**
misurata, l'impatto sull'utente, le **ore** e il documento di profondità. **Gli agenti scelgono il lavoro
dall'alto.** Quando chiudi un gap, cancellalo o accorcialo e aggiorna gli altri documenti se i numeri
cambiano.

## Stadio: gate scritto, non sentimento

Lo stadio (`pre-alpha` → `alpha` → `beta` → `1.0`) si dichiara solo se passa un **gate scritto**:

- **alpha** — i flussi principali funzionano end to end: import/ispezione di una foto, analisi AI, salvataggio
  nell'archivio, riapertura dell'archivio, esportazione del report.
- **beta** — prontezza sopra un'asticella scritta e nessun gap bloccante nel percorso principale.
- **1.0** — stabilità, gestione degli errori e copertura dei casi d'uso completi.

## Cronologia delle revisioni

Ogni documento finisce con una tabella, dal più recente:

```
| Data | Modifica | Riassunto |
|---|---|---|
| 2026-10-13 | minor | Cosa è cambiato e quale numero si è mosso |
```

Il riassunto dice **cosa è cambiato e da quale numero a quale** — non ripete l'intestazione.
