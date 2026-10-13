//! L'archivio delle analisi, dietro un'interfaccia.
//!
//! Il programma Python salva tutto in un database SQLite (`storage.py`). Qui l'archivio è descritto da
//! un `trait` e realizzato, per ora, **in memoria**: il backend SQLite arriva nella tappa T1, così il
//! resto del programma non conosce i dettagli del database.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

/// Un record di analisi: il sottoinsieme dei campi che serve a far girare l'engine.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisRecord {
    /// Assegnato dall'archivio al salvataggio; vale 0 per un record nuovo.
    pub id: i64,
    pub file_name: String,
    /// Il percorso normalizzato: identifica la foto e impedisce i duplicati.
    pub source_key: String,
    pub score: Option<i32>,
    pub tags: Vec<String>,
}

/// Cosa può andare storto parlando con l'archivio.
#[derive(Debug, PartialEq, Eq)]
pub enum ArchiveError {
    /// Il record richiesto non esiste.
    NotFound(i64),
    /// Il database non si è potuto aprire o interrogare.
    Backend(String),
}

impl std::fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "record {id} non trovato"),
            Self::Backend(what) => write!(f, "errore dell'archivio: {what}"),
        }
    }
}

impl std::error::Error for ArchiveError {}

/// Un archivio di record di analisi.
pub trait Archive {
    /// Salva un record. Se `source_key` esiste già, lo aggiorna e restituisce l'id esistente.
    fn save(&mut self, record: AnalysisRecord) -> Result<i64, ArchiveError>;

    fn get(&self, id: i64) -> Result<&AnalysisRecord, ArchiveError>;

    /// Tutti i record, dal più recente (qui semplicemente in ordine di inserto).
    fn history(&self) -> Vec<&AnalysisRecord>;

    fn count(&self) -> usize;
}

/// Archivio in memoria: regge i test e serve da base finché non c'è SQLite.
#[derive(Debug, Default)]
pub struct MemoryArchive {
    next_id: i64,
    records: Vec<AnalysisRecord>,
}

impl MemoryArchive {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            records: Vec::new(),
        }
    }
}

impl Archive for MemoryArchive {
    fn save(&mut self, mut record: AnalysisRecord) -> Result<i64, ArchiveError> {
        if let Some(existing) = self
            .records
            .iter_mut()
            .find(|r| r.source_key == record.source_key)
        {
            record.id = existing.id;
            *existing = record;
            Ok(existing.id)
        } else {
            record.id = self.next_id;
            self.next_id = self.next_id.saturating_add(1);
            let id = record.id;
            self.records.push(record);
            Ok(id)
        }
    }

    fn get(&self, id: i64) -> Result<&AnalysisRecord, ArchiveError> {
        self.records
            .iter()
            .find(|r| r.id == id)
            .ok_or(ArchiveError::NotFound(id))
    }

    fn history(&self) -> Vec<&AnalysisRecord> {
        self.records.iter().rev().collect()
    }

    fn count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(file: &str) -> AnalysisRecord {
        AnalysisRecord {
            id: 0,
            file_name: file.to_string(),
            source_key: format!("/foto/{file}"),
            score: Some(72),
            tags: vec!["paesaggio".to_string()],
        }
    }

    #[test]
    fn save_then_get_returns_the_same_record() {
        let mut archive = MemoryArchive::new();
        let id = archive.save(record("a.jpg")).expect("salvataggio");
        assert_eq!(archive.count(), 1);
        let stored = archive.get(id).expect("lettura");
        assert_eq!(stored.file_name, "a.jpg");
        assert_eq!(stored.score, Some(72));
    }

    #[test]
    fn saving_the_same_photo_twice_updates_instead_of_duplicating() {
        let mut archive = MemoryArchive::new();
        let first = archive.save(record("a.jpg")).expect("primo");
        let mut again = record("a.jpg");
        again.score = Some(90);
        let second = archive.save(again).expect("secondo");
        assert_eq!(first, second);
        assert_eq!(archive.count(), 1);
        assert_eq!(archive.get(first).expect("lettura").score, Some(90));
    }

    #[test]
    fn a_missing_record_is_an_error_not_a_panic() {
        let archive = MemoryArchive::new();
        assert_eq!(archive.get(7), Err(ArchiveError::NotFound(7)));
    }
}
