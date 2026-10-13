//! L'integrazione con i modelli: provider, richieste, risposte, errori.
//!
//! Nel programma Python questa logica stava dentro `app.py` (`call_critic`, `call_chatgpt`, prompt e
//! schemi). Qui è isolata dietro un unico `trait`: la parte applicativa non sa se dietro c'è Ollama o
//! ChatGPT. Le chiamate di rete vere arrivano nella tappa T3.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

/// Il provider scelto nel Setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Ollama,
    OpenAi,
}

/// Cosa può andare storto parlando con un modello. Ricalca gli errori che il programma Python
/// distingueva già (`MissingModel`, `NonVisionModel`, `OllamaMemoryError`, rete, risposta storta).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiError {
    /// Il modello chiesto non è installato.
    MissingModel(String),
    /// Il modello scelto non è un modello vision.
    NonVisionModel(String),
    /// Nessun modello vision è riuscito ad avviarsi per memoria insufficiente; sono elencati i provati.
    OutOfMemory(Vec<String>),
    /// Errore di rete o timeout.
    Network(String),
    /// La risposta non è il JSON atteso e non è stato possibile recuperarla.
    BadResponse(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingModel(model) => write!(f, "modello non installato: {model}"),
            Self::NonVisionModel(model) => write!(f, "il modello non è vision: {model}"),
            Self::OutOfMemory(tried) => {
                write!(f, "memoria insufficiente; modelli provati: {}", tried.join(", "))
            }
            Self::Network(what) => write!(f, "errore di rete: {what}"),
            Self::BadResponse(what) => write!(f, "risposta non valida: {what}"),
        }
    }
}

impl std::error::Error for AiError {}

/// Una richiesta di analisi: un prompt e le immagini codificate (vista completa + crop).
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisRequest {
    pub prompt: String,
    pub images: Vec<Vec<u8>>,
}

/// La risposta del modello: il testo JSON e il modello che ha davvero risposto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisResponse {
    pub model: String,
    pub json: String,
}

/// Un valutatore. La stessa interfaccia per Ollama e per OpenAI.
pub trait Critic {
    fn analyze(&self, request: &AnalysisRequest) -> Result<AnalysisResponse, AiError>;
}

/// Un valutatore finto, per i test dell'engine senza toccare la rete.
#[derive(Debug, Clone)]
pub struct FakeCritic {
    pub model: String,
    pub response: Result<String, AiError>,
}

impl Critic for FakeCritic {
    fn analyze(&self, _request: &AnalysisRequest) -> Result<AnalysisResponse, AiError> {
        self.response.clone().map(|json| AnalysisResponse {
            model: self.model.clone(),
            json,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fake_critic_returns_a_canned_response() {
        let critic = FakeCritic {
            model: "finto".to_string(),
            response: Ok("{}".to_string()),
        };
        let request = AnalysisRequest {
            prompt: "analizza".to_string(),
            images: vec![],
        };
        let response = critic.analyze(&request).expect("risposta");
        assert_eq!(response.model, "finto");
        assert_eq!(response.json, "{}");
    }

    #[test]
    fn a_memory_failure_lists_the_models_it_tried() {
        let error = AiError::OutOfMemory(vec!["qwen2.5vl:3b".to_string(), "gemma3:4b".to_string()]);
        let text = error.to_string();
        assert!(text.contains("qwen2.5vl:3b"));
        assert!(text.contains("gemma3:4b"));
    }
}
