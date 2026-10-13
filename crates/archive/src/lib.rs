//! L'archivio delle analisi: **SQLite**, dietro un'interfaccia, più un archivio in memoria per i test.
//!
//! Lo schema è nuovo (non quello del programma Python): l'import dal vecchio database è una tappa a
//! parte. Le scritture sono affidabili e un database corrotto è un errore, mai un panic.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

use std::path::Path;

/// Un record di analisi.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisRecord {
    /// Assegnato dall'archivio al salvataggio; vale 0 per un record nuovo.
    pub id: i64,
    pub file_name: String,
    /// Il percorso normalizzato: identifica la foto e impedisce i duplicati.
    pub source_key: String,
    pub score: Option<i32>,
    pub tags: Vec<String>,
    pub model: Option<String>,
    /// Il report completo, così come l'ha restituito il modello.
    pub report_json: String,
    /// La fotocamera usata (per le statistiche).
    pub camera: Option<String>,
    /// Gli ISO dello scatto (per le statistiche).
    pub iso: Option<u32>,
    /// La valutazione a stelle (0–5), se l'utente l'ha data.
    pub rating: Option<i32>,
}

impl AnalysisRecord {
    /// Un record nuovo, senza id.
    pub fn new(file_name: impl Into<String>, source_key: impl Into<String>) -> Self {
        Self {
            id: 0,
            file_name: file_name.into(),
            source_key: source_key.into(),
            score: None,
            tags: Vec::new(),
            model: None,
            report_json: String::new(),
            camera: None,
            iso: None,
            rating: None,
        }
    }
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

    fn get(&self, id: i64) -> Result<AnalysisRecord, ArchiveError>;

    /// Un record dalla sua fonte (percorso), se c'è.
    fn find(&self, source_key: &str) -> Result<Option<AnalysisRecord>, ArchiveError>;

    /// Tutti i record, dal più recente.
    fn history(&self) -> Result<Vec<AnalysisRecord>, ArchiveError>;

    fn count(&self) -> usize;
}

fn backend(error: rusqlite::Error) -> ArchiveError {
    ArchiveError::Backend(error.to_string())
}

/// Le colonne lette, sempre nello stesso ordine.
const COLUMNS: &str = "id, file_name, source_key, score, tags, model, report_json, camera, iso, rating";

/// L'archivio su un file SQLite.
pub struct SqliteArchive {
    conn: rusqlite::Connection,
}

impl SqliteArchive {
    /// Apre (o crea) il database e prepara lo schema.
    pub fn open(path: &Path) -> Result<Self, ArchiveError> {
        let conn = rusqlite::Connection::open(path).map_err(backend)?;
        let archive = Self { conn };
        archive.init()?;
        Ok(archive)
    }

    fn init(&self) -> Result<(), ArchiveError> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS analyses (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    file_name TEXT NOT NULL,
                    source_key TEXT NOT NULL UNIQUE,
                    score INTEGER,
                    tags TEXT NOT NULL DEFAULT '[]',
                    model TEXT,
                    report_json TEXT NOT NULL DEFAULT '',
                    camera TEXT,
                    iso INTEGER,
                    rating INTEGER
                );
                CREATE TABLE IF NOT EXISTS settings (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );",
            )
            .map_err(backend)?;
        // Colonne aggiunte dopo la prima versione: idempotente, l'errore "esiste già" si ignora.
        let _ = self
            .conn
            .execute("ALTER TABLE analyses ADD COLUMN camera TEXT", []);
        let _ = self
            .conn
            .execute("ALTER TABLE analyses ADD COLUMN iso INTEGER", []);
        let _ = self
            .conn
            .execute("ALTER TABLE analyses ADD COLUMN rating INTEGER", []);
        Ok(())
    }

    /// Legge un'impostazione.
    pub fn get_setting(&self, key: &str) -> Option<String> {
        self.conn
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get::<_, String>(0)
            })
            .ok()
    }

    /// Scrive un'impostazione.
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), ArchiveError> {
        self.conn
            .execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                rusqlite::params![key, value],
            )
            .map_err(backend)?;
        Ok(())
    }

    fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<AnalysisRecord> {
        let tags_json: String = row.get(4)?;
        Ok(AnalysisRecord {
            id: row.get(0)?,
            file_name: row.get(1)?,
            source_key: row.get(2)?,
            score: row.get(3)?,
            tags: serde_json::from_str(&tags_json).unwrap_or_default(),
            model: row.get(5)?,
            report_json: row.get(6)?,
            camera: row.get(7)?,
            iso: row.get(8)?,
            rating: row.get(9)?,
        })
    }

    /// Importa le analisi dal database del programma Python (schema diverso), con una traduzione
    /// best-effort. Restituisce quante righe ha importato.
    pub fn import_python(&mut self, python_db: &Path) -> Result<usize, ArchiveError> {
        let source =
            rusqlite::Connection::open_with_flags(python_db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .map_err(backend)?;
        let mut statement = source
            .prepare(
                "SELECT file_name, file_path, source_key, score, tags_json, model, report_json FROM analyses",
            )
            .map_err(backend)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<f64>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                ))
            })
            .map_err(backend)?;
        let mut imported = 0usize;
        for row in rows {
            let (file_name, file_path, source_key, score, tags_json, model, report_json) =
                row.map_err(backend)?;
            let key = source_key
                .or(file_path)
                .or_else(|| file_name.clone())
                .unwrap_or_default();
            if key.is_empty() {
                continue;
            }
            let mut record = AnalysisRecord::new(file_name.unwrap_or_default(), key);
            record.score = score.map(|value| value.round() as i32);
            record.tags = tags_json
                .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
                .unwrap_or_default();
            record.model = model;
            record.report_json = report_json.unwrap_or_default();
            self.save(record)?;
            imported += 1;
        }
        Ok(imported)
    }
}

impl Archive for SqliteArchive {
    fn save(&mut self, record: AnalysisRecord) -> Result<i64, ArchiveError> {
        let tags = serde_json::to_string(&record.tags).map_err(|e| ArchiveError::Backend(e.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO analyses (file_name, source_key, score, tags, model, report_json, camera, iso, rating)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(source_key) DO UPDATE SET
                    file_name = excluded.file_name,
                    score = excluded.score,
                    tags = excluded.tags,
                    model = excluded.model,
                    report_json = excluded.report_json,
                    camera = excluded.camera,
                    iso = excluded.iso,
                    rating = excluded.rating",
                rusqlite::params![
                    record.file_name,
                    record.source_key,
                    record.score,
                    tags,
                    record.model,
                    record.report_json,
                    record.camera,
                    record.iso,
                    record.rating
                ],
            )
            .map_err(backend)?;
        self.conn
            .query_row(
                "SELECT id FROM analyses WHERE source_key = ?1",
                [&record.source_key],
                |row| row.get::<_, i64>(0),
            )
            .map_err(backend)
    }

    fn get(&self, id: i64) -> Result<AnalysisRecord, ArchiveError> {
        let sql = format!("SELECT {COLUMNS} FROM analyses WHERE id = ?1");
        self.conn
            .query_row(&sql, [id], SqliteArchive::row_to_record)
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => ArchiveError::NotFound(id),
                other => backend(other),
            })
    }

    fn find(&self, source_key: &str) -> Result<Option<AnalysisRecord>, ArchiveError> {
        let sql = format!("SELECT {COLUMNS} FROM analyses WHERE source_key = ?1");
        match self
            .conn
            .query_row(&sql, [source_key], SqliteArchive::row_to_record)
        {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(backend(error)),
        }
    }

    fn history(&self) -> Result<Vec<AnalysisRecord>, ArchiveError> {
        let sql = format!("SELECT {COLUMNS} FROM analyses ORDER BY id DESC");
        let mut statement = self.conn.prepare(&sql).map_err(backend)?;
        let rows = statement
            .query_map([], SqliteArchive::row_to_record)
            .map_err(backend)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(backend)?);
        }
        Ok(out)
    }

    fn count(&self) -> usize {
        self.conn
            .query_row("SELECT COUNT(*) FROM analyses", [], |row| row.get::<_, i64>(0))
            .map(|n| n.max(0) as usize)
            .unwrap_or(0)
    }
}

/// Archivio in memoria, per i test.
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

    fn get(&self, id: i64) -> Result<AnalysisRecord, ArchiveError> {
        self.records
            .iter()
            .find(|r| r.id == id)
            .cloned()
            .ok_or(ArchiveError::NotFound(id))
    }

    fn find(&self, source_key: &str) -> Result<Option<AnalysisRecord>, ArchiveError> {
        Ok(self.records.iter().find(|r| r.source_key == source_key).cloned())
    }

    fn history(&self) -> Result<Vec<AnalysisRecord>, ArchiveError> {
        Ok(self.records.iter().rev().cloned().collect())
    }

    fn count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(file: &str, score: i32) -> AnalysisRecord {
        let mut record = AnalysisRecord::new(file, format!("/foto/{file}"));
        record.score = Some(score);
        record.tags = vec!["paesaggio".to_string()];
        record.model = Some("gemma3:4b".to_string());
        record.report_json = "{\"punteggio_totale\":72}".to_string();
        record
    }

    fn round_trip(archive: &mut dyn Archive) {
        let id = archive.save(record("a.jpg", 72)).expect("salvataggio");
        let stored = archive.get(id).expect("lettura");
        assert_eq!(stored.file_name, "a.jpg");
        assert_eq!(stored.score, Some(72));
        assert_eq!(stored.tags, vec!["paesaggio".to_string()]);
        assert_eq!(stored.model.as_deref(), Some("gemma3:4b"));

        // salvare la stessa foto due volte aggiorna, non duplica
        let updated = archive.save(record("a.jpg", 90)).expect("aggiornamento");
        assert_eq!(id, updated);
        assert_eq!(archive.count(), 1);
        assert_eq!(archive.get(id).expect("lettura").score, Some(90));

        // lo storico ha il record
        assert_eq!(archive.history().expect("storico").len(), 1);

        // un id assente è un errore, non un panic
        assert_eq!(archive.get(9999), Err(ArchiveError::NotFound(9999)));
    }

    #[test]
    fn the_memory_archive_round_trips() {
        round_trip(&mut MemoryArchive::new());
    }

    #[test]
    fn the_sqlite_archive_round_trips() {
        let dir = std::env::temp_dir().join(format!("postcritica-archive-{}", std::process::id()));
        let _ = std::fs::remove_file(&dir);
        let mut archive = SqliteArchive::open(&dir).expect("apertura");
        round_trip(&mut archive);
        let _ = std::fs::remove_file(&dir);
    }
}
