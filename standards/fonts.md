# Standard: font

> **Standard di PhotoInsight Critic.** Adattato dallo standard condiviso `fonts` di `craftrules`
> (repository privato), riscritto in italiano perché il progetto resti autosufficiente.

Regola di base: **i file dei font non stanno nel repository del prodotto.** Appesantiscono ogni clone e
ogni build, e mescolano la licenza del codice con quella dei font. Vivono in un **repository dei font
separato** (come `storytold/craft-fonts` per LightCraft) e si consumano da lì.

## Come si consumano

- Si consumano tramite un **input di build opzionale** (una variabile d'ambiente che punta al clone dei
  font). Se l'input non è impostato, **tutto compila, testa e gira**, ma i testi che richiedono quei font
  appaiono senza glifi (riquadri vuoti). I test che servono quei font si saltano senza di essi.
- Nelle **release** i font si includono sempre, e si spediscono **anche i testi delle loro licenze**.

## Licenze

- Sono ammessi solo font con licenza aperta compatibile: **OFL**, oppure pubblico dominio / CC0 / una
  licenza permissiva (MIT / Apache-2.0 / BSD / ISC).
- Per ogni font spedito, includi il **testo della licenza** (per esempio `OFL-*.txt`) accanto ai font, e
  registra la provenienza in `assets/ATTRIBUTION.md`.
- **Non copiare i font di altri prodotti**, nemmeno se hanno una licenza aperta.

## Glifi e fallback

- Definisci i font per **script**: un font principale e fallback per gli altri alfabeti, scelti per script.
- Se un testo non ha glifi, quello è un bug di copertura, non una resa accettabile.

## Icone e immagini

- Le icone si disegnano come **vettori in codice** (originali) e si registrano come lavoro originale in
  `assets/ATTRIBUTION.md`.
- Non si usano icone, screenshot o immagini di altri prodotti, **nemmeno come segnaposto temporaneo**.
- Le foto di esempio e i riferimenti per i benchmark sono materiale aperto o generato (vedi lo standard
  `test-corpora`).

## Cosa non fare

- **Non committare mai** file di font (`.ttf`, `.otf`, `.woff2`, …) nel repository del prodotto.
- Nel dubbio sulla licenza di un asset: **non usarlo**.
