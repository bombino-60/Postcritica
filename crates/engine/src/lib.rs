//! I comandi del programma e l'orchestrazione.
//!
//! Tiene insieme archivio e valutatore e offre le azioni che la UI invocherà per id. Nel programma
//! Python questa logica stava dentro le rotte di `app.py`; qui è indipendente dalla UI, così la si può
//! testare senza finestra.

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

use postcritica_ai::{AiError, AnalysisRequest, Critic};
use postcritica_archive::ArchiveError;
use postcritica_metadata::{self, MetadataError};
use postcritica_raster::{self, RasterError};

// Tipi condivisi ri-esportati, così il livello UI dipende solo dall'engine.
pub use postcritica_ai::{OllamaCritic, Report};
pub use postcritica_analysis::VisualProfile;
pub use postcritica_archive::{AnalysisRecord, Archive, SqliteArchive};
pub use postcritica_metadata::PhotoMetadata;
pub use postcritica_raster::{rgba_bytes, Rgba8};

/// Il risultato dell'ispezione di una foto: metadati, profilo visivo e miniatura.
pub struct Inspection {
    pub metadata: PhotoMetadata,
    pub profile: VisualProfile,
    /// Le dimensioni dichiarate dal file (0, 0 se non leggibili).
    pub original: (u32, u32),
    pub thumb: Rgba8,
}

/// Ispeziona una foto: legge i metadati, decodifica una miniatura e ne calcola il profilo visivo.
///
/// Non va mai in panic: un file illeggibile esce come `EngineError`.
pub fn inspect_photo(path: &Path, max_edge: u32) -> Result<Inspection, EngineError> {
    let metadata = postcritica_metadata::read(path).map_err(EngineError::Metadata)?;
    let original = postcritica_raster::dimensions(path).unwrap_or((0, 0));
    let thumb = postcritica_raster::decode_thumbnail(path, max_edge).map_err(EngineError::Raster)?;
    let profile = VisualProfile::of(&thumb);
    Ok(Inspection {
        metadata,
        profile,
        original,
        thumb,
    })
}

/// Il risultato di un'analisi: i pezzi che la UI mostra e che poi si salvano nell'archivio.
pub struct AnalysisOutcome {
    pub file_name: String,
    pub source_key: String,
    pub model: String,
    pub report_json: String,
    pub report: Report,
}

/// Esegue l'analisi di una foto **senza** toccare l'archivio: ispeziona, chiede al modello, interpreta.
///
/// È pensato per girare in un thread: la UI lo chiama, poi salva il risultato nell'archivio sul thread
/// principale (la connessione SQLite non si sposta fra thread).
pub fn run_analysis<C: Critic>(critic: &C, path: &Path) -> Result<AnalysisOutcome, EngineError> {
    let inspection = inspect_photo(path, 512)?;
    let image = postcritica_raster::encode_png(&inspection.thumb).map_err(EngineError::Raster)?;
    let request = AnalysisRequest {
        prompt: postcritica_ai::build_prompt(),
        images: vec![image],
    };
    let response = critic.analyze(&request).map_err(EngineError::Ai)?;
    let report = postcritica_ai::parse_report(&response.json);
    Ok(AnalysisOutcome {
        file_name: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string(),
        source_key: path.to_string_lossy().to_string(),
        model: response.model,
        report_json: response.json,
        report,
    })
}

/// Il comando `photo.analyze`: ispeziona, chiede al modello, salva il record e lo restituisce.
pub fn analyze_photo<A: Archive, C: Critic>(
    archive: &mut A,
    critic: &C,
    path: &Path,
) -> Result<AnalysisRecord, EngineError> {
    let outcome = run_analysis(critic, path)?;
    let mut record = AnalysisRecord::new(outcome.file_name, outcome.source_key);
    record.score = outcome.report.score;
    record.tags = outcome.report.tag;
    record.model = Some(outcome.model);
    record.report_json = outcome.report_json;

    let id = archive.save(record).map_err(EngineError::Archive)?;
    archive.get(id).map_err(EngineError::Archive)
}

/// L'engine: un archivio e un valutatore.
pub struct Engine<A, C> {
    archive: A,
    critic: C,
}

impl<A: Archive, C: Critic> Engine<A, C> {
    pub fn new(archive: A, critic: C) -> Self {
        Self { archive, critic }
    }

    /// L'archivio, in sola lettura, per la UI.
    pub fn archive(&self) -> &A {
        &self.archive
    }

    /// Il comando `photo.analyze` su dati già pronti (usato dai test).
    pub fn analyze(
        &mut self,
        file_name: &str,
        source_key: &str,
        prompt: &str,
        images: Vec<Vec<u8>>,
    ) -> Result<i64, EngineError> {
        let request = AnalysisRequest {
            prompt: prompt.to_string(),
            images,
        };
        let response = self.critic.analyze(&request).map_err(EngineError::Ai)?;
        let report = postcritica_ai::parse_report(&response.json);
        let mut record = AnalysisRecord::new(file_name, source_key);
        record.score = report.score;
        record.tags = report.tag;
        record.model = Some(response.model);
        record.report_json = response.json;
        self.archive.save(record).map_err(EngineError::Archive)
    }
}

/// Cosa può andare storto in un comando.
#[derive(Debug)]
pub enum EngineError {
    Archive(ArchiveError),
    Ai(AiError),
    Metadata(MetadataError),
    Raster(RasterError),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Archive(error) => write!(f, "{error}"),
            Self::Ai(error) => write!(f, "{error}"),
            Self::Metadata(error) => write!(f, "{error}"),
            Self::Raster(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for EngineError {}

#[cfg(test)]
mod tests {
    use super::*;
    use postcritica_ai::FakeCritic;
    use postcritica_archive::MemoryArchive;

    fn engine(json: &str) -> Engine<MemoryArchive, FakeCritic> {
        Engine::new(
            MemoryArchive::new(),
            FakeCritic {
                model: "finto".to_string(),
                response: Ok(json.to_string()),
            },
        )
    }

    #[test]
    fn analyze_saves_a_record_with_the_score_from_the_report() {
        let mut engine = engine("{\"punteggio_totale\": 81, \"tag\": [\"mare\"]}");
        let id = engine
            .analyze("a.jpg", "/foto/a.jpg", "analizza", vec![])
            .expect("analisi");
        let stored = engine.archive().get(id).expect("lettura");
        assert_eq!(stored.score, Some(81));
        assert_eq!(stored.tags, vec!["mare".to_string()]);
    }

    #[test]
    fn a_broken_report_does_not_panic() {
        let mut engine = engine("testo spazzatura senza numeri");
        let id = engine
            .analyze("a.jpg", "/foto/a.jpg", "analizza", vec![])
            .expect("analisi");
        assert_eq!(engine.archive().get(id).expect("lettura").score, None);
    }

    #[test]
    fn an_ai_failure_is_reported_as_an_error() {
        let mut engine = Engine::new(
            MemoryArchive::new(),
            FakeCritic {
                model: "finto".to_string(),
                response: Err(AiError::Network("giu'".to_string())),
            },
        );
        let result = engine.analyze("a.jpg", "/foto/a.jpg", "analizza", vec![]);
        assert!(matches!(result, Err(EngineError::Ai(AiError::Network(_)))));
    }
}
