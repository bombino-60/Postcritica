//! PhotoInsight Critic — applicazione desktop.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

fn main() -> eframe::Result<()> {
    photoinsight_ui_egui::run()
}
