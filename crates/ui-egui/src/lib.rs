//! L'interfaccia desktop (egui/eframe) di Postcritica.
//!
//! Il layout segue il riferimento LightCraft: barra dei menu in alto, pannello **Libreria** a sinistra,
//! **griglia** al centro, pannello **Analisi** a destra, una striscia di **moduli** all'estrema destra e
//! una **barra** in basso. La veste grafica usa la palette scura dell'app di riferimento.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};

use eframe::egui;
use postcritica_engine::{
    inspect_photo, rgba_bytes, run_analysis, AnalysisOutcome, AnalysisRecord, Archive, Inspection,
    OllamaCritic, PhotoMetadata, Rgba8, SqliteArchive, VisualProfile,
};

/// Il modello Ollama predefinito (come il programma Python).
const DEFAULT_MODEL: &str = "gemma3:4b";
/// Il file dell'archivio locale.
const ARCHIVE_FILE: &str = "postcritica.db";

/// Apre la finestra dell'app. Ritorna quando l'utente la chiude.
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 860.0])
            .with_title("Postcritica"),
        ..Default::default()
    };
    eframe::run_native(
        "Postcritica",
        options,
        Box::new(|cc| {
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(App::new(cc)))
        }),
    )
}

/// Il modulo scelto nella striscia di destra (come i moduli di Lightroom).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Module {
    Library,
    Analysis,
    Archive,
    Collections,
    Stats,
}

impl Module {
    const ALL: [Module; 5] = [
        Module::Library,
        Module::Analysis,
        Module::Archive,
        Module::Collections,
        Module::Stats,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Library => "Libreria",
            Self::Analysis => "Analisi",
            Self::Archive => "Archivio",
            Self::Collections => "Raccolte",
            Self::Stats => "Statistiche",
        }
    }
}

/// Una foto mostrata nella griglia.
struct Photo {
    name: String,
    /// Il file sul disco; `None` per le immagini della demo.
    path: Option<PathBuf>,
    thumb: egui::TextureHandle,
    inspection: Inspection,
}

/// Il report di un'analisi, pronto da mostrare.
struct AnalysisReport {
    score: Option<i32>,
    tags: Vec<String>,
    strengths: Vec<String>,
    weaknesses: Vec<String>,
    summary: Option<String>,
    model: String,
}

/// Un messaggio dai thread (lettura cartella, analisi).
enum Msg {
    Photo(Box<Photo>),
    Progress(usize, usize),
    Error(String),
    Done,
    Analyzed(Box<AnalysisOutcome>),
}

/// Lo stato dell'app.
struct App {
    photos: Vec<Photo>,
    selected: Option<usize>,
    reader: Option<Receiver<Msg>>,
    analyzer: Option<Receiver<Msg>>,
    status: String,
    model: String,
    module: Module,
    thumb_size: f32,
    folder: Option<String>,
    archive: Option<SqliteArchive>,
    reports: HashMap<String, AnalysisReport>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let _ = cc;
        let archive = SqliteArchive::open(Path::new(ARCHIVE_FILE)).ok();
        let status = match &archive {
            Some(store) => format!("Pronto. Archivio: {ARCHIVE_FILE} ({} analisi).", store.count()),
            None => format!("Pronto, ma non riesco ad aprire l'archivio {ARCHIVE_FILE}."),
        };
        Self {
            photos: Vec::new(),
            selected: None,
            reader: None,
            analyzer: None,
            status,
            model: DEFAULT_MODEL.to_string(),
            module: Module::Library,
            thumb_size: 150.0,
            folder: None,
            archive,
            reports: HashMap::new(),
        }
    }

    /// Raccoglie i messaggi dei thread e aggiorna lo stato.
    fn drain(&mut self, ctx: &egui::Context) {
        let mut messages = Vec::new();
        let mut reader_done = false;
        if let Some(receiver) = &self.reader {
            loop {
                match receiver.try_recv() {
                    Ok(message) => messages.push(message),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        reader_done = true;
                        break;
                    }
                }
            }
        }
        let mut analyzer_done = false;
        if let Some(receiver) = &self.analyzer {
            loop {
                match receiver.try_recv() {
                    Ok(message) => messages.push(message),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        analyzer_done = true;
                        break;
                    }
                }
            }
        }
        for message in messages {
            match message {
                Msg::Photo(photo) => self.photos.push(*photo),
                Msg::Progress(done, total) => self.status = format!("{done}/{total}…"),
                Msg::Error(message) => self.status = message,
                Msg::Done => {
                    reader_done = true;
                    self.status = format!("{} foto.", self.photos.len());
                }
                Msg::Analyzed(outcome) => self.store_outcome(*outcome),
            }
        }
        if reader_done {
            self.reader = None;
        }
        if analyzer_done {
            self.analyzer = None;
        }
        if self.reader.is_some() || self.analyzer.is_some() {
            ctx.request_repaint();
        }
    }

    /// Salva l'analisi nell'archivio e la rende visibile.
    fn store_outcome(&mut self, outcome: AnalysisOutcome) {
        if let Some(store) = &mut self.archive {
            let mut record = AnalysisRecord::new(outcome.file_name.clone(), outcome.source_key.clone());
            record.score = outcome.report.score;
            record.tags = outcome.report.tag.clone();
            record.model = Some(outcome.model.clone());
            record.report_json = outcome.report_json.clone();
            if let Err(error) = store.save(record) {
                self.status = format!("{error}");
            }
        }
        self.reports.insert(
            outcome.source_key.clone(),
            AnalysisReport {
                score: outcome.report.score,
                tags: outcome.report.tag.clone(),
                strengths: outcome.report.strengths.clone(),
                weaknesses: outcome.report.weaknesses.clone(),
                summary: outcome.report.sintesi.clone(),
                model: outcome.model.clone(),
            },
        );
        self.status = format!("Analisi completata per {}.", outcome.file_name);
    }

    fn start_folder(&mut self, ctx: &egui::Context, dir: PathBuf) {
        self.photos.clear();
        self.selected = None;
        self.folder = dir.file_name().and_then(|n| n.to_str()).map(str::to_string);
        let (sender, receiver) = mpsc::channel();
        self.reader = Some(receiver);
        self.status = format!("Leggo {}…", dir.display());
        let ctx = ctx.clone();
        std::thread::spawn(move || read_folder(&dir, &ctx, &sender));
    }

    fn start_analysis(&mut self, ctx: &egui::Context) {
        let Some(photo) = self.selected.and_then(|index| self.photos.get(index)) else {
            self.status = "Seleziona prima una foto.".to_string();
            return;
        };
        let Some(path) = photo.path.clone() else {
            self.status =
                "Le immagini della demo non hanno un file da analizzare: apri una cartella.".to_string();
            return;
        };
        let name = photo.name.clone();
        let model = self.model.clone();
        let (sender, receiver) = mpsc::channel();
        self.analyzer = Some(receiver);
        self.status = format!("Analizzo {name} con {model}… (può richiedere qualche minuto)");
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let critic = OllamaCritic::new(model);
            match run_analysis(&critic, &path) {
                Ok(outcome) => {
                    let _ = sender.send(Msg::Analyzed(Box::new(outcome)));
                }
                Err(error) => {
                    let _ = sender.send(Msg::Error(format!("Analisi non riuscita: {error}")));
                }
            }
            ctx.request_repaint();
        });
    }

    fn load_demo(&mut self, ctx: &egui::Context) {
        self.photos.clear();
        self.selected = None;
        self.folder = Some("(demo)".to_string());
        let (width, height) = (360usize, 240usize);
        for index in 0..12usize {
            let mut data = Vec::with_capacity(width * height);
            for y in 0..height {
                for x in 0..width {
                    let fx = x as f64 / width as f64;
                    let fy = y as f64 / height as f64;
                    let phase = index as f64 / 12.0;
                    let stripe = ((fx * 9.0 + phase * 8.0).sin() * 0.5 + 0.5).powi(2);
                    data.push([
                        (fx * 255.0) as u8,
                        (fy * 255.0) as u8,
                        (stripe * 255.0) as u8,
                        255,
                    ]);
                }
            }
            let Some(image) = Rgba8::new(width, height, data) else {
                continue;
            };
            let profile = VisualProfile::of(&image);
            let thumb = make_texture(ctx, &format!("demo-{index}"), &image);
            self.photos.push(Photo {
                name: format!("demo-{index:02}.png"),
                path: None,
                thumb,
                inspection: Inspection {
                    metadata: PhotoMetadata {
                        shot_at: None,
                        ..Default::default()
                    },
                    profile,
                    original: (width as u32, height as u32),
                    thumb: image,
                },
            });
        }
        self.status = format!("{} foto nella demo.", self.photos.len());
    }

    fn selected_key(&self) -> Option<String> {
        let photo = self.photos.get(self.selected?)?;
        photo.path.as_ref().map(|path| path.to_string_lossy().to_string())
    }

    /// I tag più frequenti fra i report, con il loro conteggio.
    fn tag_counts(&self) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for report in self.reports.values() {
            for tag in &report.tags {
                *counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        let mut list: Vec<(String, usize)> = counts.into_iter().collect();
        list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        list
    }

    /// La griglia delle foto, raggruppate per giorno di scatto.
    fn library(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Tutte le foto");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.weak(format!("{} foto", self.photos.len()));
            });
        });
        ui.separator();
        if self.photos.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Nessuna foto. Apri una cartella dal menu «File», o premi «Demo».");
            });
            return;
        }

        let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
        for (index, photo) in self.photos.iter().enumerate() {
            let day = photo_day(photo);
            match groups.iter_mut().find(|(label, _)| *label == day) {
                Some((_, list)) => list.push(index),
                None => groups.push((day, vec![index])),
            }
        }

        let mut clicked = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (label, indices) in &groups {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(format!("{label} · {} foto", indices.len())).strong());
                ui.add_space(2.0);
                ui.horizontal_wrapped(|ui| {
                    for &index in indices {
                        if let Some(photo) = self.photos.get(index) {
                            if thumbnail(ui, photo, self.selected == Some(index), self.thumb_size).clicked() {
                                clicked = Some(index);
                            }
                        }
                    }
                });
            }
        });
        if let Some(index) = clicked {
            self.selected = Some(index);
        }
    }

    /// L'elenco delle analisi salvate nell'archivio.
    fn archive_view(&self, ui: &mut egui::Ui) {
        let records = self
            .archive
            .as_ref()
            .and_then(|store| store.history().ok())
            .unwrap_or_default();
        ui.horizontal(|ui| {
            ui.heading("Archivio e cronologia");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.weak(format!("{} analisi", records.len()));
            });
        });
        ui.separator();
        if records.is_empty() {
            ui.label("Nessuna analisi salvata. Apri una foto e premi «Analizza».");
            return;
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            for record in &records {
                ui.horizontal(|ui| {
                    let score = record
                        .score
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "—".to_string());
                    ui.label(
                        egui::RichText::new(format!("{score:>3}"))
                            .strong()
                            .color(score_colour(record.score)),
                    );
                    ui.label(&record.file_name);
                });
                ui.weak(record.tags.join(" · "));
                ui.separator();
            }
        });
    }

    /// Il pannello di destra: metadati, profilo visivo e report della foto scelta.
    fn inspector(&self, ui: &mut egui::Ui) {
        let Some(photo) = self.selected.and_then(|index| self.photos.get(index)) else {
            ui.add_space(6.0);
            ui.strong("Analisi");
            ui.separator();
            ui.weak("Seleziona una foto.");
            return;
        };
        if ui
            .add_enabled(photo.path.is_some(), egui::Button::new("Analizza con Ollama"))
            .clicked()
        {
            ui.ctx()
                .data_mut(|data| data.insert_temp(egui::Id::new("analizza"), true));
        }
        ui.add_space(6.0);
        ui.heading(&photo.name);
        let (width, height) = photo.inspection.original;
        ui.label(format!("{width} × {height} px"));
        if let Some(day) = photo.inspection.metadata.shot_at.clone() {
            ui.weak(day);
        }
        ui.add_space(6.0);
        ui.separator();
        ui.strong("Metadati");
        let metadata = &photo.inspection.metadata;
        field(ui, "Fotocamera", metadata.camera.clone());
        field(ui, "Obiettivo", metadata.lens.clone());
        field(ui, "ISO", metadata.iso.map(|v| v.to_string()));
        field(ui, "Apertura", metadata.aperture.map(|v| format!("f/{v:.1}")));
        field(ui, "Tempo", metadata.shutter.clone());
        field(ui, "Focale", metadata.focal_mm.map(|v| format!("{v:.0} mm")));
        ui.add_space(6.0);
        ui.separator();
        ui.strong("Profilo visivo");
        let profile = &photo.inspection.profile;
        ui.label(format!("Luminosità: {:.0}%", profile.brightness * 100.0));
        ui.label(format!("Contrasto: {:.0}%", profile.contrast * 100.0));
        ui.add_space(6.0);
        ui.separator();
        ui.strong("Valutazione");

        let report = self.selected_key().and_then(|key| self.reports.get(&key));
        match report {
            Some(report) => {
                let score = report
                    .score
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "—".to_string());
                ui.label(
                    egui::RichText::new(format!("Punteggio: {score} / 100"))
                        .heading()
                        .color(score_colour(report.score)),
                );
                ui.weak(format!("Modello: {}", report.model));
                if let Some(summary) = &report.summary {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(summary).italics());
                }
                if !report.tags.is_empty() {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Tag").strong());
                    ui.label(report.tags.join(" · "));
                }
                list(ui, "Punti di forza", &report.strengths, true);
                list(ui, "Punti di debolezza", &report.weaknesses, false);
            }
            None => {
                ui.weak("Nessuna analisi. Premi «Analizza con Ollama».");
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.drain(&ctx);

        // La richiesta di analisi arriva dal pannello di destra: la leggo qui, dove ho `&mut self`.
        let requested = ui
            .ctx()
            .data_mut(|data| data.remove_temp::<bool>(egui::Id::new("analizza")))
            .unwrap_or(false);
        if requested {
            self.start_analysis(&ctx);
        }

        egui::Panel::top("menu").show(ui, |ui| {
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Apri cartella…").clicked() {
                        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                            self.start_folder(&ctx, dir);
                        }
                        ui.close();
                    }
                    if ui.button("Demo").clicked() {
                        self.load_demo(&ctx);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Esci").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Modifica", |ui| {
                    ui.weak("(in arrivo)");
                });
                ui.menu_button("Vista", |ui| {
                    ui.weak("(in arrivo)");
                });
                ui.separator();
                ui.strong("Postcritica");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.weak("Ollama:");
                    ui.add(egui::TextEdit::singleline(&mut self.model).desired_width(150.0));
                });
            });
            ui.add_space(3.0);
        });

        egui::Panel::bottom("barra").show(ui, |ui| {
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ui.weak("Dimensione miniature");
                ui.add(egui::Slider::new(&mut self.thumb_size, 90.0..=280.0).show_value(false));
                ui.separator();
                for _ in 0..5 {
                    ui.label(egui::RichText::new("★").color(egui::Color32::from_rgb(0x3a, 0x42, 0x4e)));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.weak(&self.status);
                });
            });
            ui.add_space(3.0);
        });

        egui::Panel::left("libreria")
            .default_size(248.0)
            .min_size(190.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(4.0);
                    ui.strong("Libreria");
                    ui.separator();
                    egui::CollapsingHeader::new("Le mie foto")
                        .default_open(true)
                        .show(ui, |ui| {
                            let _ = ui.selectable_label(self.module == Module::Library, "Tutte le foto");
                            ui.weak(format!("  {}", self.photos.len()));
                        });
                    egui::CollapsingHeader::new("Cartelle")
                        .default_open(true)
                        .show(ui, |ui| {
                            match &self.folder {
                                Some(name) => {
                                    ui.label(name);
                                }
                                None => {
                                    ui.weak("Nessuna cartella aperta");
                                }
                            }
                            if ui.button("Apri cartella…").clicked() {
                                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                                    self.start_folder(&ctx, dir);
                                }
                            }
                        });
                    egui::CollapsingHeader::new("Tag")
                        .default_open(true)
                        .show(ui, |ui| {
                            let tags = self.tag_counts();
                            if tags.is_empty() {
                                ui.weak("Nessun tag: analizza una foto");
                            }
                            for (tag, count) in tags.iter().take(20) {
                                ui.horizontal(|ui| {
                                    ui.label(format!("• {tag}"));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.weak(count.to_string());
                                    });
                                });
                            }
                        });
                    egui::CollapsingHeader::new("Archivio")
                        .default_open(false)
                        .show(ui, |ui| {
                            let count = self.archive.as_ref().map_or(0, |store| store.count());
                            let _ = ui.selectable_label(
                                self.module == Module::Archive,
                                format!("Analisi salvate: {count}"),
                            );
                        });
                });
            });

        egui::Panel::right("moduli").exact_size(42.0).show(ui, |ui| {
            ui.add_space(6.0);
            for module in Module::ALL {
                if module_button(ui, self.module == module, module).clicked() {
                    self.module = module;
                }
                ui.add_space(4.0);
            }
        });

        egui::Panel::right("pannello")
            .min_size(300.0)
            .default_size(340.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(4.0);
                    self.inspector(ui);
                });
            });

        egui::CentralPanel::default().show(ui, |ui| match self.module {
            Module::Library | Module::Analysis => self.library(ui),
            Module::Archive => self.archive_view(ui),
            Module::Collections => placeholder(
                ui,
                "Raccolte",
                "Le raccolte manuali e la ricerca semantica arriveranno qui.",
            ),
            Module::Stats => placeholder(ui, "Statistiche", "I grafici di distribuzione arriveranno qui."),
        });
    }
}

/// Applica la palette scura dell'app di riferimento.
fn apply_theme(ctx: &egui::Context) {
    let background = egui::Color32::from_rgb(0x0b, 0x0d, 0x10);
    let panel = egui::Color32::from_rgb(0x13, 0x17, 0x1c);
    let line = egui::Color32::from_rgb(0x2a, 0x32, 0x3d);
    let text = egui::Color32::from_rgb(0xed, 0xf1, 0xf5);
    let muted = egui::Color32::from_rgb(0x92, 0x9c, 0xa8);
    let blue = egui::Color32::from_rgb(0x5d, 0xa9, 0xff);
    let raised = egui::Color32::from_rgb(0x1a, 0x1f, 0x26);
    let hovered = egui::Color32::from_rgb(0x24, 0x2b, 0x35);

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = panel;
    visuals.window_fill = panel;
    visuals.extreme_bg_color = background;
    visuals.override_text_color = Some(text);
    visuals.window_stroke = egui::Stroke::new(1.0, line);
    visuals.hyperlink_color = blue;
    visuals.selection.bg_fill = blue;
    visuals.selection.stroke = egui::Stroke::new(1.0, text);
    visuals.widgets.noninteractive.bg_fill = panel;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, line);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, muted);
    visuals.widgets.inactive.weak_bg_fill = raised;
    visuals.widgets.inactive.bg_fill = raised;
    visuals.widgets.hovered.weak_bg_fill = hovered;
    visuals.widgets.hovered.bg_fill = hovered;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, blue);
    visuals.widgets.active.weak_bg_fill = blue;
    visuals.widgets.active.bg_fill = blue;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, blue);
    ctx.set_visuals(visuals);
}

/// Un pulsante quadrato della striscia dei moduli, con l'icona disegnata in codice (lavoro originale).
fn module_button(ui: &mut egui::Ui, selected: bool, module: Module) -> egui::Response {
    let size = egui::vec2(34.0, 34.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let visuals = ui.visuals().clone();
    let (bg, colour) = if selected {
        (visuals.selection.bg_fill, egui::Color32::WHITE)
    } else if response.hovered() {
        (
            visuals.widgets.hovered.bg_fill,
            visuals.widgets.hovered.fg_stroke.color,
        )
    } else {
        (
            egui::Color32::TRANSPARENT,
            visuals.widgets.inactive.fg_stroke.color,
        )
    };
    if bg != egui::Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, 4.0, bg);
    }
    draw_module_icon(ui.painter(), rect.shrink(9.0), colour, module);
    response.on_hover_text(module.label())
}

/// Disegna l'icona di un modulo con semplici tratti.
fn draw_module_icon(painter: &egui::Painter, rect: egui::Rect, colour: egui::Color32, module: Module) {
    let stroke = egui::Stroke::new(1.5, colour);
    match module {
        Module::Library => {
            let half = rect.width() / 2.0 - 1.0;
            for (dx, dy) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
                let min = egui::pos2(rect.min.x + dx * (half + 2.0), rect.min.y + dy * (half + 2.0));
                painter.rect_stroke(
                    egui::Rect::from_min_size(min, egui::vec2(half, half)),
                    1.0,
                    stroke,
                    egui::StrokeKind::Inside,
                );
            }
        }
        Module::Analysis => {
            painter.circle_stroke(rect.center() - egui::vec2(2.0, 2.0), rect.width() * 0.32, stroke);
            painter.line_segment([rect.center() + egui::vec2(1.0, 1.0), rect.max], stroke);
        }
        Module::Archive => {
            for i in 0..3 {
                let y = rect.min.y + i as f32 * (rect.height() / 2.0);
                painter.line_segment([egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)], stroke);
            }
        }
        Module::Collections => {
            painter.rect_stroke(
                egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * 0.7, rect.height() * 0.7)),
                1.0,
                stroke,
                egui::StrokeKind::Inside,
            );
            painter.rect_stroke(
                egui::Rect::from_min_size(
                    rect.min + egui::vec2(rect.width() * 0.3, rect.height() * 0.3),
                    egui::vec2(rect.width() * 0.7, rect.height() * 0.7),
                ),
                1.0,
                stroke,
                egui::StrokeKind::Inside,
            );
        }
        Module::Stats => {
            let bars = [(0.30, 0.55), (0.60, 1.0), (0.90, 0.75)];
            let width = rect.width() / 5.0;
            for (x, height) in bars.iter() {
                let left = rect.min.x + *x * rect.width() - width / 2.0;
                let bar = egui::Rect::from_min_max(
                    egui::pos2(left, rect.max.y - height * rect.height()),
                    egui::pos2(left + width, rect.max.y),
                );
                painter.rect_filled(bar, 1.0, colour);
            }
        }
    }
}

fn placeholder(ui: &mut egui::Ui, title: &str, note: &str) {
    ui.add_space(6.0);
    ui.heading(title);
    ui.separator();
    ui.weak(note);
}

fn score_colour(score: Option<i32>) -> egui::Color32 {
    match score {
        Some(value) if value >= 75 => egui::Color32::from_rgb(0x54, 0xc9, 0x85),
        Some(value) if value >= 50 => egui::Color32::from_rgb(0xe8, 0xb6, 0x5a),
        Some(_) => egui::Color32::from_rgb(0xef, 0x6a, 0x68),
        None => egui::Color32::from_rgb(0x92, 0x9c, 0xa8),
    }
}

fn list(ui: &mut egui::Ui, title: &str, items: &[String], positive: bool) {
    if items.is_empty() {
        return;
    }
    ui.add_space(4.0);
    ui.label(egui::RichText::new(title).strong());
    let colour = if positive {
        egui::Color32::from_rgb(0x54, 0xc9, 0x85)
    } else {
        egui::Color32::from_rgb(0xef, 0x6a, 0x68)
    };
    for item in items {
        ui.label(egui::RichText::new(format!("• {item}")).color(colour));
    }
}

/// Il giorno di scatto, per raggruppare la griglia.
fn photo_day(photo: &Photo) -> String {
    match photo.inspection.metadata.shot_at.as_deref() {
        Some(value) if value.chars().count() >= 10 => value.chars().take(10).collect(),
        Some(value) if !value.trim().is_empty() => value.to_string(),
        _ => "Senza data".to_string(),
    }
}

fn thumbnail(ui: &mut egui::Ui, photo: &Photo, selected: bool, size: f32) -> egui::Response {
    let target = egui::vec2(size, size * 0.72);
    let image = egui::Image::new(&photo.thumb)
        .fit_to_exact_size(target)
        .sense(egui::Sense::click());
    let response = ui.add(image);
    if selected {
        ui.painter().rect_stroke(
            response.rect,
            3.0,
            egui::Stroke::new(2.5, egui::Color32::from_rgb(0x5d, 0xa9, 0xff)),
            egui::StrokeKind::Inside,
        );
    }
    response.on_hover_text(&photo.name)
}

fn field(ui: &mut egui::Ui, label: &str, value: Option<String>) {
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        ui.weak(value.unwrap_or_else(|| "—".to_string()));
    });
}

fn make_texture(ctx: &egui::Context, name: &str, image: &Rgba8) -> egui::TextureHandle {
    let bytes = rgba_bytes(image);
    let color = egui::ColorImage::from_rgba_unmultiplied([image.width(), image.height()], &bytes);
    ctx.load_texture(name.to_string(), color, egui::TextureOptions::LINEAR)
}

fn read_folder(dir: &Path, ctx: &egui::Context, sender: &Sender<Msg>) {
    let mut paths = Vec::new();
    collect_images(dir, &mut paths, 0);
    let total = paths.len();
    if total == 0 {
        let _ = sender.send(Msg::Error(format!("Nessuna immagine in {}", dir.display())));
        let _ = sender.send(Msg::Done);
        ctx.request_repaint();
        return;
    }
    for (index, path) in paths.iter().enumerate() {
        match inspect_photo(path, 256) {
            Ok(inspection) => {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?")
                    .to_string();
                let thumb = make_texture(ctx, &path.display().to_string(), &inspection.thumb);
                let _ = sender.send(Msg::Photo(Box::new(Photo {
                    name,
                    path: Some(path.clone()),
                    thumb,
                    inspection,
                })));
            }
            Err(error) => {
                let _ = sender.send(Msg::Error(format!("{}: {error}", path.display())));
            }
        }
        let _ = sender.send(Msg::Progress(index + 1, total));
        ctx.request_repaint();
    }
    let _ = sender.send(Msg::Done);
    ctx.request_repaint();
}

fn collect_images(dir: &Path, out: &mut Vec<PathBuf>, depth: u8) {
    const MAX_DEPTH: u8 = 5;
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_images(&path, out, depth + 1);
        } else if is_image(&path) {
            out.push(path);
        }
    }
}

fn is_image(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "jpg" | "jpeg" | "png" | "tif" | "tiff" | "webp" | "bmp" | "gif"
    )
}
