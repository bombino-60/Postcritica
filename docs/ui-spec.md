# Linee guida della UI

> **Ultima revisione:** 2026-10-13 · **Ultimo aggiornamento:** 2026-10-13 · **Modifica:** minor (font, fondi e selettori dal riferimento) · **Obiettivo:** la stessa veste del riferimento LightCraft

Come deve apparire e comportarsi l'interfaccia, ricavato **osservando la schermata del riferimento**
(LightCraft). I valori stanno nel modulo `crates/ui-egui/src/theme.rs`; questo documento li spiega.

## Font

- **Un solo font di interfaccia: Inter** (come il riferimento). Regular per il testo, **SemiBold per i
  titoli**. I file stanno in `assets/fonts/` (`Inter-Regular.ttf`, `Inter-SemiBold.ttf`), con la licenza
  OFL accanto (`OFL-Inter.txt`) e la voce in `assets/ATTRIBUTION.md`. I font predefiniti di egui restano
  come **ripiego** per i glifi che Inter non copre.
- **Misure:**
  | Testo | Font | Dimensione |
  |---|---|---|
  | Corpo (etichette, campi, pulsanti) | Inter Regular | 15 px |
  | Piccolo (note, conteggi) | Inter Regular | 12,5 px |
  | Titoli di pannello e sezione | Inter SemiBold | 20 px |
  | Monospaziato (numeri, percorsi) | monospace | 14 px |

## Fondi e colori

I colori sono quelli del tema del riferimento (grigi neutri, nessuna dominante):

| Ruolo | Esadecimale | Dove |
|---|---|---|
| `chrome` | `#2d2d2d` | barra dei menu, pannelli laterali, barra in basso, striscia dei moduli |
| `grid_bg` | `#0f0f0f` | sfondo dietro le celle della griglia e la foto |
| `canvas` / `cell` | `#1c1c1c` | tela della foto e celle |
| `inset` | `#232323` | fondi incassati (istogramma) |
| `field` | `#222222` | campi di testo e caselle |
| `button` | `#242424` | pulsanti a riposo |
| `button_border` | `#4c4c4c` | bordo dei pulsanti |
| `hover` / `pressed` | `#3a3a3a` / `#3f3f3f` | pulsanti al passaggio e premuti |
| `text` | `#e2e2e2` | testo evidenziato |
| `text_label` | `#bcbcbc` | testo normale |
| `text_dim` | `#8e8e8e` | testo attenuato |
| `icon` | `#9a9a9a` | icone della chrome |
| `accent` | `#0165dd` | selezione, accenti |
| `reject` / `caution` / `positive` | `#e04a4a` / `#e3a83c` / `#58b05c` | rosso, ambra, verde dei giudizi |

## Struttura (regioni)

Dall'esterno verso l'interno:

1. **Barra dei menu** (`chrome`) — File · Modifica · Vista · Foto; **campo di ricerca** al centro; a destra
   il modello e le icone.
2. **Barra dei comandi** (`chrome`, in basso, 30–46 px) — viste (griglia/foto singola), «Analizza», stelle,
   avanzamento, dimensione delle miniature, stato.
3. **Striscia di miniature** (`chrome`, sopra la barra, 72–118 px) — la foto scelta è evidenziata
   dall'**accento**.
4. **Pannello Libreria** (`chrome`, 210–380 px) con la **linguetta «Libreria»** in cima: Le mie foto,
   Cartelle, Tag, Archivio.
5. **Striscia dei moduli** (a destra, 48 px) — icone disegnate in codice: Libreria, Analisi, Archivio,
   Raccolte, Statistiche, Setup.
6. **Pannello Strumenti** (`chrome`, 340–560 px) con la **linguetta «Strumenti»**: istogramma, metadati,
   profilo visivo, report, valutazione a stelle, «Esporta PDF».
7. **Centro** (`grid_bg`) — intestazione (nella foto singola: nome · data · dimensioni) e poi la **foto
   grande** oppure la **griglia** raggruppata per data.

## Selettori e controlli

- **Pulsanti:** fondo `button`, bordo `button_border`, raggio 4; al passaggio `hover`, premuti `pressed`.
- **Campi di testo:** fondo `field`, testo `text`; usano l'`extreme_bg_color` del tema.
- **Menu a tendina (selettori):** fondo `button`, bordo `button_border`, freccia `▼`; quelli attivi usano
  l'`accent`. Sono i selettori del Setup (modello Ollama), del tipo di file e simili.
- **Linguette di pannello:** una etichetta selezionabile in cima al pannello.
- **Voci selezionate:** fondo `accent` al 60 %, bordo `accent`.
- **Spaziature:** 8×6 tra gli elementi, 10×5 dentro i pulsanti, altezza minima di tocco 26 px.

## Verifica

Il tema sta in `crates/ui-egui/src/theme.rs` (`Tokens`, `install_fonts`, `apply`). Chi cambia un colore, un
font o una misura lo fa lì, non nel codice delle viste.
