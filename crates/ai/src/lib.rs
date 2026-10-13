//! L'integrazione con i modelli: provider, richieste, risposte, errori.
//!
//! Qui c'è il client di **Ollama** (analisi vision in locale) e il tipo del report. Il client di OpenAI
//! arriva più avanti, dietro la stessa interfaccia.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

use std::time::Duration;

use base64::Engine as _;
use serde::{Deserialize, Serialize};

/// Il provider scelto nel Setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Ollama,
    OpenAi,
}

/// Cosa può andare storto parlando con un modello.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiError {
    /// Il modello chiesto non è installato.
    MissingModel(String),
    /// Il modello scelto non è un modello vision.
    NonVisionModel(String),
    /// Nessun modello vision è riuscito ad avviarsi per memoria insufficiente; sono elencati i provati.
    OutOfMemory(Vec<String>),
    /// Errore di rete, timeout, o servizio non raggiungibile.
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
            Self::Network(what) => write!(f, "{what}"),
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

/// Un valutatore: la stessa interfaccia per Ollama e per OpenAI.
pub trait Critic {
    fn analyze(&self, request: &AnalysisRequest) -> Result<AnalysisResponse, AiError>;
}

/// Il report strutturato che il modello restituisce.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Report {
    #[serde(rename = "punteggio_totale", default)]
    pub score: Option<i32>,
    #[serde(default)]
    pub tag: Vec<String>,
    #[serde(rename = "punti_di_forza", default)]
    pub strengths: Vec<String>,
    #[serde(rename = "punti_di_debolezza", default)]
    pub weaknesses: Vec<String>,
    #[serde(default)]
    pub sintesi: Option<String>,
}

/// Interpreta il report JSON. Un testo storto diventa un report vuoto, mai un panic.
pub fn parse_report(json: &str) -> Report {
    serde_json::from_str(json).unwrap_or_default()
}

/// Il prompt inviato al modello, in italiano, con i campi attesi.
pub fn build_prompt() -> String {
    "Sei un critico fotografico esperto. Osserva la foto e rispondi SOLO con un oggetto JSON, senza \
     testo attorno, con questi campi: \"punteggio_totale\" (numero intero da 0 a 100), \"tag\" (elenco di \
     parole singole in italiano, con spazi e mai underscore, al massimo 12), \"punti_di_forza\" (elenco di \
     frasi brevi), \"punti_di_debolezza\" (elenco di frasi brevi), \"sintesi\" (una frase). Valuta tecnica, \
     composizione e messaggio. Rispondi in italiano."
        .to_string()
}

/// Il client di Ollama per l'analisi vision.
#[derive(Debug, Clone)]
pub struct OllamaCritic {
    pub url: String,
    pub model: String,
}

impl OllamaCritic {
    /// Un client con l'indirizzo locale predefinito.
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            url: "http://localhost:11434".to_string(),
            model: model.into(),
        }
    }

    /// I modelli installati in Ollama, per far scegliere nella UI.
    pub fn available_models(url: &str) -> Result<Vec<String>, AiError> {
        let endpoint = format!("{}/api/tags", url.trim_end_matches('/'));
        match ureq::get(&endpoint).timeout(Duration::from_secs(15)).call() {
            Ok(response) => {
                let value: serde_json::Value = response
                    .into_json()
                    .map_err(|e| AiError::BadResponse(e.to_string()))?;
                let models = value
                    .get("models")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                Ok(models)
            }
            Err(ureq::Error::Transport(transport)) => Err(AiError::Network(format!(
                "Ollama non risponde su {url} ({transport}). È avviato?"
            ))),
            Err(ureq::Error::Status(code, _)) => Err(AiError::Network(format!("Ollama ha risposto {code}"))),
        }
    }
}

/// Lo schema JSON che vincola l'output del modello (riduce i loop e le risposte fuori formato).
pub fn report_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "punteggio_totale": { "type": "integer" },
            "tag": { "type": "array", "items": { "type": "string" } },
            "punti_di_forza": { "type": "array", "items": { "type": "string" } },
            "punti_di_debolezza": { "type": "array", "items": { "type": "string" } },
            "sintesi": { "type": "string" }
        },
        "required": ["punteggio_totale", "tag", "punti_di_forza", "punti_di_debolezza", "sintesi"]
    })
}

impl Critic for OllamaCritic {
    fn analyze(&self, request: &AnalysisRequest) -> Result<AnalysisResponse, AiError> {
        let images: Vec<String> = request
            .images
            .iter()
            .map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes))
            .collect();
        let body = serde_json::json!({
            "model": self.model,
            "prompt": request.prompt,
            "images": images,
            "stream": false,
            "format": report_schema(),
            "options": {
                "temperature": 0.2,
                "seed": 42,
                "num_ctx": 8192,
                "num_predict": 1024,
                "repeat_penalty": 1.15,
                "repeat_last_n": 256
            }
        });
        let endpoint = format!("{}/api/generate", self.url.trim_end_matches('/'));
        match ureq::post(&endpoint)
            .timeout(Duration::from_secs(600))
            .send_json(&body)
        {
            Ok(response) => {
                let value: serde_json::Value = response
                    .into_json()
                    .map_err(|e| AiError::BadResponse(e.to_string()))?;
                let text = value
                    .get("response")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                if text.is_empty() {
                    return Err(AiError::BadResponse("il modello ha risposto vuoto".to_string()));
                }
                let model = value
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&self.model)
                    .to_string();
                Ok(AnalysisResponse { model, json: text })
            }
            Err(ureq::Error::Status(code, response)) => {
                let body = response.into_string().unwrap_or_default();
                Err(classify_http(code, &body, &self.model))
            }
            Err(ureq::Error::Transport(transport)) => Err(AiError::Network(format!(
                "non riesco a contattare Ollama su {} ({transport}). È avviato?",
                self.url
            ))),
        }
    }
}

/// Il client di OpenAI (ChatGPT) per l'analisi vision.
#[derive(Debug, Clone)]
pub struct OpenAiCritic {
    pub url: String,
    pub model: String,
    pub key: String,
}

impl OpenAiCritic {
    /// Un client con l'endpoint predefinito.
    pub fn new(model: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            url: "https://api.openai.com/v1/chat/completions".to_string(),
            model: model.into(),
            key: key.into(),
        }
    }
}

impl Critic for OpenAiCritic {
    fn analyze(&self, request: &AnalysisRequest) -> Result<AnalysisResponse, AiError> {
        if self.key.trim().is_empty() {
            return Err(AiError::Network("manca la chiave API di OpenAI".to_string()));
        }
        let mut content = vec![serde_json::json!({ "type": "text", "text": request.prompt })];
        for image in &request.images {
            let encoded = base64::engine::general_purpose::STANDARD.encode(image);
            content.push(serde_json::json!({
                "type": "image_url",
                "image_url": { "url": format!("data:image/png;base64,{encoded}"), "detail": "high" }
            }));
        }
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": "Sei un critico fotografico esperto." },
                { "role": "user", "content": content }
            ],
            "response_format": { "type": "json_object" },
            "max_completion_tokens": 1200
        });
        match ureq::post(&self.url)
            .set("Authorization", &format!("Bearer {}", self.key))
            .timeout(Duration::from_secs(600))
            .send_json(&body)
        {
            Ok(response) => {
                let value: serde_json::Value = response
                    .into_json()
                    .map_err(|e| AiError::BadResponse(e.to_string()))?;
                let text = value
                    .get("choices")
                    .and_then(|choices| choices.get(0))
                    .and_then(|choice| choice.get("message"))
                    .and_then(|message| message.get("content"))
                    .and_then(|content| content.as_str())
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                if text.is_empty() {
                    return Err(AiError::BadResponse("OpenAI ha risposto vuoto".to_string()));
                }
                Ok(AnalysisResponse {
                    model: self.model.clone(),
                    json: text,
                })
            }
            Err(ureq::Error::Status(code, response)) => {
                let message = response.into_string().unwrap_or_default();
                Err(classify_openai(code, &message))
            }
            Err(ureq::Error::Transport(transport)) => Err(AiError::Network(format!(
                "non riesco a contattare OpenAI ({transport})"
            ))),
        }
    }
}

/// Traduce una risposta HTTP di errore di OpenAI in un `AiError` parlante.
fn classify_openai(code: u16, body: &str) -> AiError {
    match code {
        401 | 403 => AiError::Network("chiave API di OpenAI non valida".to_string()),
        429 => AiError::Network("OpenAI ha risposto: troppe richieste".to_string()),
        _ => AiError::Network(format!("OpenAI ha risposto {code}: {}", body.trim())),
    }
}

/// Traduce una risposta HTTP di errore in un `AiError` parlante.
fn classify_http(code: u16, body: &str, model: &str) -> AiError {
    let lower = body.to_ascii_lowercase();
    if code == 404 || lower.contains("not found") {
        AiError::MissingModel(model.to_string())
    } else if lower.contains("memory") {
        AiError::OutOfMemory(vec![model.to_string()])
    } else {
        AiError::Network(format!("Ollama ha risposto {code}: {}", body.trim()))
    }
}

/// Un valutatore finto, per i test senza rete.
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
    fn a_report_is_parsed_from_italian_keys() {
        let report = parse_report(
            "{\"punteggio_totale\": 81, \"tag\": [\"mare\", \"tramonto\"], \
             \"punti_di_forza\": [\"luce\"], \"punti_di_debolezza\": [\"orizzonte\"], \"sintesi\": \"buona\"}",
        );
        assert_eq!(report.score, Some(81));
        assert_eq!(report.tag, vec!["mare".to_string(), "tramonto".to_string()]);
        assert_eq!(report.strengths, vec!["luce".to_string()]);
        assert_eq!(report.weaknesses, vec!["orizzonte".to_string()]);
        assert_eq!(report.sintesi.as_deref(), Some("buona"));
    }

    #[test]
    fn a_broken_report_is_empty_not_a_panic() {
        assert_eq!(parse_report("testo spazzatura"), Report::default());
        assert_eq!(parse_report(""), Report::default());
    }

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
        assert_eq!(critic.analyze(&request).expect("risposta").model, "finto");
    }
}
