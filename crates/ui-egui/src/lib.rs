//! L'interfaccia desktop (egui/eframe) di Postcritica.
//!
//! Prima versione usabile: si apre una cartella di foto (o si carica una demo), si vedono le miniature in
//! una griglia e, selezionandone una, i metadati e il profilo visivo. Premendo «Analizza», la foto viene
//! inviata a Ollama e il report compare a destra, salvato nell'archivio SQLite.

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
            .with_inner_size([1180.0, 760.0])
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
    archive: Option<SqliteArchive>,
    reports: HashMap<String, AnalysisReport>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let _ = cc;
        let archive = SqliteArchive::open(Path::new(ARCHIVE_FILE)).ok();
        let status = match &archive {
            Some(store) => format!(
                "Pronto. Archivio: {ARCHIVE_FILE} ({} analisi salvate).",
                store.count()
            ),
            None => format!("Pronto, ma non riesco ad aprire l'archivio {ARCHIVE_FILE}."),
        };
        Self {
            photos: Vec::new(),
            selected: None,
            reader: None,
            analyzer: None,
            status,
            model: DEFAULT_MODEL.to_string(),
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

    /// Salva l'analisi nell'archivio e la rende visibile nella UI.
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

    /// Avvia la lettura di una cartella in un thread separato.
    fn start_folder(&mut self, ctx: &egui::Context, dir: PathBuf) {
        self.photos.clear();
        self.selected = None;
        let (sender, receiver) = mpsc::channel();
        self.reader = Some(receiver);
        self.status = format!("Leggo {}…", dir.display());
        let ctx = ctx.clone();
        std::thread::spawn(move || read_folder(&dir, &ctx, &sender));
    }

    /// Avvia l'analisi della foto scelta, in un thread separato.
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

    /// Costruisce dodici foto procedurali, così c'è sempre qualcosa da guardare.
    fn load_demo(&mut self, ctx: &egui::Context) {
        self.photos.clear();
        self.selected = None;
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
                    metadata: PhotoMetadata::default(),
                    profile,
                    original: (width as u32, height as u32),
                    thumb: image,
                },
            });
        }
        self.status = format!("{} foto nella demo.", self.photos.len());
    }

    /// La chiave della foto scelta, per cercarne il report.
    fn selected_key(&self) -> Option<String> {
        let photo = self.photos.get(self.selected?)?;
        photo.path.as_ref().map(|path| path.to_string_lossy().to_string())
    }

    /// Il pannello di destra: metadati, profilo visivo e report.
    fn inspector(&self, ui: &mut egui::Ui) {
        let Some(photo) = self.selected.and_then(|index| self.photos.get(index)) else {
            ui.label("Seleziona una foto.");
            return;
        };
        ui.heading(&photo.name);
        ui.add_space(4.0);
        let (width, height) = photo.inspection.original;
        ui.label(format!("Dimensioni: {width} × {height} px"));
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
                ui.add_space(2.0);
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
                ui.weak("Nessuna analisi. Seleziona una foto e premi «Analizza».");
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.drain(&ctx);

        egui::Panel::top("barra").show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button("Apri cartella…").clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        self.start_folder(&ctx, dir);
                    }
                }
                if ui.button("Demo").clicked() {
                    self.load_demo(&ctx);
                }
                ui.separator();
                if ui
                    .add_enabled(self.selected.is_some(), egui::Button::new("Analizza"))
                    .clicked()
                {
                    self.start_analysis(&ctx);
                }
                ui.label("Modello:");
                ui.add(egui::TextEdit::singleline(&mut self.model).desired_width(150.0));
                ui.separator();
                ui.weak(&self.status);
            });
            ui.add_space(4.0);
        });

        egui::Panel::right("ispettore")
            .min_size(320.0)
            .default_size(360.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.inspector(ui));
            });

        egui::CentralPanel::default().show(ui, |ui| {
            if self.photos.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("Nessuna foto. Apri una cartella o premi «Demo».");
                });
                return;
            }
            let mut clicked = None;
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (index, photo) in self.photos.iter().enumerate() {
                        if thumbnail(ui, photo, self.selected == Some(index)).clicked() {
                            clicked = Some(index);
                        }
                    }
                });
            });
            if let Some(index) = clicked {
                self.selected = Some(index);
            }
        });
    }
}

/// Applica la palette della tua app (sfondo, pannelli, accenti) al tema di egui.
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

/// Il colore del punteggio, dal rosso al verde.
fn score_colour(score: Option<i32>) -> egui::Color32 {
    match score {
        Some(value) if value >= 75 => egui::Color32::from_rgb(0x54, 0xc9, 0x85),
        Some(value) if value >= 50 => egui::Color32::from_rgb(0xe8, 0xb6, 0x5a),
        Some(_) => egui::Color32::from_rgb(0xef, 0x6a, 0x68),
        None => egui::Color32::from_rgb(0x92, 0x9c, 0xa8),
    }
}

/// Un elenco puntato, verde per i punti di forza e rosso per le debolezze.
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

/// Disegna una miniatura cliccabile, con la cornice quando è quella scelta.
fn thumbnail(ui: &mut egui::Ui, photo: &Photo, selected: bool) -> egui::Response {
    let size = egui::vec2(148.0, 108.0);
    let image = egui::Image::new(&photo.thumb)
        .fit_to_exact_size(size)
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

/// Una riga "etichetta: valore", con un trattino quando il valore manca.
fn field(ui: &mut egui::Ui, label: &str, value: Option<String>) {
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        ui.weak(value.unwrap_or_else(|| "—".to_string()));
    });
}

/// Crea una texture da un'immagine RGBA8.
fn make_texture(ctx: &egui::Context, name: &str, image: &Rgba8) -> egui::TextureHandle {
    let bytes = rgba_bytes(image);
    let color = egui::ColorImage::from_rgba_unmultiplied([image.width(), image.height()], &bytes);
    ctx.load_texture(name.to_string(), color, egui::TextureOptions::LINEAR)
}

/// Legge una cartella in un thread: per ogni immagine, ispeziona e manda il risultato alla UI.
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

/// Raccoglie i percorsi delle immagini, con un tetto alla profondità (l'input è ostile).
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

/// Se il percorso ha un'estensione fra quelle che sappiamo leggere.
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
