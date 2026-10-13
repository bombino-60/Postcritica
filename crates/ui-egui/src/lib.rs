//! L'interfaccia desktop (egui/eframe) di Postcritica.
//!
//! Livello più alto del programma: conosce l'engine, non conosce il database né i provider. Qui c'è
//! solo lo scheletro della finestra; le viste (libreria, dettaglio, analisi, impostazioni, raccolte,
//! statistiche, batch) arrivano nella tappa T7.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

use eframe::egui;

/// Apre la finestra dell'app. Ritorna quando l'utente la chiude.
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Postcritica",
        options,
        Box::new(|_cc| Ok(Box::new(PostcriticaApp::default()))),
    )
}

/// Lo stato dell'app. Vuoto finché non ci sono le viste.
#[derive(Default)]
struct PostcriticaApp {}

impl eframe::App for PostcriticaApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("Postcritica");
        ui.add_space(8.0);
        ui.label("Scheletro dell'applicazione. La libreria e le viste arrivano nella tappa T7.");
    }
}
