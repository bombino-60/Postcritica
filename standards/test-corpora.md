# Standard: corpora e file di test

> **Standard di Postcritica.** Adattato dallo standard condiviso `test-corpora` di `craftrules`
> (repository privato), riscritto in italiano perché il progetto resti autosufficiente.

I test che usano file reali (foto, RAW, JPEG con EXIF, XMP) hanno bisogno di quei file — ma **i media
grandi non si committano mai** e non si usano file di provenienza incerta. Postcritica produce
inoltre dati pesanti (il database, gli output) che non devono finire nel versionamento.

## Cosa va fuori dal versionamento

- Il **database** (`*.db`, `*.db-wal`, `*.db-shm`), la cartella degli **output**, i **media** dei corpus,
  il target di cargo e ogni file generato.
- Tutto questo vive in `.gitignore`. **Mai committare il database o i media.**

## File di test ammessi nel repository

- Un file di test **piccolo** (poche centinaia di KB), con **licenza aperta** e provenienza registrata,
  può stare nel repository, accanto ai test.
- Preferisci la **generazione**: dove puoi, costruisci l'immagine di test in codice (pixel in memoria, un
  JPEG con EXIF scritti a mano, un XMP minimo). Niente download, niente licenze da tracciare.
- I riferimenti per i benchmark stanno nel repository **solo** se piccoli e di licenza aperta con
  provenienza registrata.

## File di test che si scaricano

- I file più grandi o i corpus reali stanno in una cartella **in `.gitignore`** (per esempio `corpus/`) e
  si scaricano da fonti aperte.
- Il download è **fissato e verificato**: URL/commit esatto e **sha256**; un comando del tooling
  (`cargo xtask corpus`) scarica, verifica e mette in cache in modo riproducibile.
- Registra **origine e licenza** di ogni corpus in un manifest accanto ad esso.

## Come i test usano i file

- I test scrivono in una cartella temporanea e non lasciano residui nel repository.
- Se un corpus necessario manca (rete spenta, download non eseguito), il test **si salta** con un
  messaggio chiaro — non fallisce e non finge di essere passato.

## Perché

Mantenere i media fuori dal repository tiene i cloni leggeri, evita di mescolare licenze di terzi con
quella del progetto, e rende i test riproducibili senza dipendere da file che nessuno sa più da dove
vengano.
