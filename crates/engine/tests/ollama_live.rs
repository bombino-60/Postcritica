//! Test d'integrazione che parla davvero con Ollama.
//!
//! È **ignorato** di default: richiede Ollama in locale (con un modello vision). Si esegue a mano con
//! `cargo test -p postcritica-engine --test ollama_live -- --ignored --nocapture`.

use postcritica_engine::{run_analysis, OllamaCritic};

#[test]
#[ignore = "richiede Ollama in locale con un modello vision"]
fn analyzes_a_real_photo_with_ollama() {
    // Un'immagine di prova, generata qui: niente da scaricare.
    let path = std::env::temp_dir().join("postcritica-live.png");
    let image = image::RgbImage::from_fn(96, 64, |x, y| image::Rgb([(x * 2) as u8, (y * 3) as u8, 120]));
    image.save(&path).expect("scrivo il png di prova");

    let critic = OllamaCritic::new("gemma3:4b");
    let outcome = run_analysis(&critic, &path).expect("analisi con Ollama");

    println!(
        "modello: {} | totale: {:?} | tecnica {} · composizione {} · messaggio {} | tag: {:?}",
        outcome.model,
        outcome.report.score,
        outcome.report.tecnica.unwrap_or(-1),
        outcome.report.composizione.unwrap_or(-1),
        outcome.report.messaggio.unwrap_or(-1),
        outcome.report.tag,
    );

    let _ = std::fs::remove_file(&path);
}
