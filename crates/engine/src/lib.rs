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

use postcritica_ai::{AiError, AnalysisRequest, Critic};
use postcritica_archive::{AnalysisRecord, Archive, ArchiveError};

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

    /// Il comando `photo.analyze`: chiede al modello e salva il record.
    ///
    /// Non va mai in panic: un errore del modello o dell'archivio esce come `EngineError`.
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
        let record = AnalysisRecord {
            id: 0,
            file_name: file_name.to_string(),
            source_key: source_key.to_string(),
            score: parse_score(&response.json),
            tags: Vec::new(),
        };
        self.archive.save(record).map_err(EngineError::Archive)
    }
}

/// Estrae `punteggio_totale` da un report JSON in modo tollerante.
///
/// Il parsing vero arriverà con l'engine completo; qui l'importante è che un input storto non faccia
/// andare in panic (niente indicizzazione, niente `unwrap`).
fn parse_score(json: &str) -> Option<i32> {
    let after_key = json.split("\"punteggio_totale\"").nth(1)?;
    let digits: String = after_key
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse::<i32>().ok()
}

/// Cosa può andare storto in un comando.
#[derive(Debug)]
pub enum EngineError {
    Archive(ArchiveError),
    Ai(AiError),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Archive(error) => write!(f, "{error}"),
            Self::Ai(error) => write!(f, "{error}"),
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
        let mut engine = engine("{\"punteggio_totale\": 81}");
        let id = engine
            .analyze("a.jpg", "/foto/a.jpg", "analizza", vec![])
            .expect("analisi");
        let stored = engine.archive().get(id).expect("lettura");
        assert_eq!(stored.score, Some(81));
    }

    #[test]
    fn a_report_without_a_score_still_saves() {
        let mut engine = engine("{\"nota\": \"nessun punteggio\"}");
        let id = engine
            .analyze("a.jpg", "/foto/a.jpg", "analizza", vec![])
            .expect("analisi");
        assert_eq!(engine.archive().get(id).expect("lettura").score, None);
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
