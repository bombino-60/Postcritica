//! L'interfaccia desktop (egui/eframe) di Postcritica.
//!
//! Il layout segue il riferimento LightCraft: barra dei menu in alto, pannello **Libreria** a sinistra,
//! **griglia** al centro, pannello **Analisi** a destra, una striscia di **moduli** all'estrema destra e
//! una **barra** in basso. Colori e misure vengono dal modulo [`theme`], copiati dal riferimento.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

mod control;
mod theme;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

use eframe::egui;
use postcritica_engine::{
    inspect_photo, rgba_bytes, run_analysis, AnalysisOutcome, AnalysisRecord, Archive, Inspection,
    OllamaCritic, OpenAiCritic, PhotoMetadata, Rgba8, SqliteArchive, VisualProfile,
};
use theme::Tokens;

/// Il modello Ollama predefinito (come il programma Python).
const DEFAULT_MODEL: &str = "gemma3:4b";
/// Il modello OpenAI predefinito.
const DEFAULT_OPENAI_MODEL: &str = "gpt-4o";
/// Il file dell'archivio locale.
const ARCHIVE_FILE: &str = "postcritica.db";
/// La porta del canale di controllo locale.
const CONTROL_PORT: u16 = 7010;
/// L'indirizzo locale di Ollama.
const OLLAMA_URL: &str = "http://localhost:11434";

/// Il formato di esportazione dell'archivio.
#[derive(Debug, Clone, Copy)]
enum ExportFormat {
    Csv,
    Json,
    JsonL,
}

/// Apre la finestra dell'app. Ritorna quando l'utente la chiude.
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1520.0, 940.0])
            .with_min_inner_size([1180.0, 720.0])
            .with_title("Postcritica"),
        ..Default::default()
    };
    eframe::run_native(
        "Postcritica",
        options,
        Box::new(|cc| {
            theme::install_fonts(&cc.egui_ctx);
            theme::apply(&cc.egui_ctx);
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
    Setup,
}

impl Module {
    const ALL: [Module; 6] = [
        Module::Library,
        Module::Analysis,
        Module::Archive,
        Module::Collections,
        Module::Stats,
        Module::Setup,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Library => "Libreria",
            Self::Analysis => "Analisi",
            Self::Archive => "Archivio",
            Self::Collections => "Raccolte",
            Self::Stats => "Statistiche",
            Self::Setup => "Setup",
        }
    }
}

/// Il provider di analisi scelto nel Setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderChoice {
    Ollama,
    ChatGpt,
}

/// La vista della libreria: griglia di miniature o foto singola grande.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewKind {
    Grid,
    Loupe,
}

impl ProviderChoice {
    fn key(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::ChatGpt => "chatgpt",
        }
    }
}

/// Una foto mostrata nella griglia.
struct Photo {
    name: String,
    path: Option<PathBuf>,
    thumb: egui::TextureHandle,
    inspection: Inspection,
}

/// Il report di un'analisi, pronto da mostrare.
struct AnalysisReport {
    score: Option<i32>,
    tecnica: Option<i32>,
    composizione: Option<i32>,
    messaggio: Option<i32>,
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
    AnalyzedError(String),
}

/// Lo stato dell'app.
struct App {
    photos: Vec<Photo>,
    selected: Option<usize>,
    reader: Option<Receiver<Msg>>,
    analyzer: Option<Receiver<Msg>>,
    status: String,
    model: String,
    provider: ProviderChoice,
    openai_model: String,
    api_key: String,
    module: Module,
    thumb_size: f32,
    folder: Option<String>,
    archive: Option<SqliteArchive>,
    reports: HashMap<String, AnalysisReport>,
    /// Vero mentre un'analisi è in corso.
    analyzing: bool,
    /// Le foto in attesa di analisi (batch).
    queue: Vec<PathBuf>,
    queue_total: usize,
    queue_done: usize,
    /// Il tag scelto nella vista Raccolte.
    selected_tag: Option<String>,
    /// Il canale di controllo locale.
    control: Option<Receiver<control::Command>>,
    snapshot: Option<Arc<Mutex<control::Snapshot>>>,
    /// I modelli Ollama installati, per il selettore del Setup.
    ollama_models: Vec<String>,
    models_receiver: Option<Receiver<Vec<String>>>,
    models_requested: bool,
    /// La vista della libreria e il testo cercato.
    view: ViewKind,
    search: String,
    /// L'anteprima grande della foto scelta, per la vista a foto singola.
    preview: Option<egui::TextureHandle>,
    preview_key: Option<String>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let _ = cc;
        let archive = SqliteArchive::open(Path::new(ARCHIVE_FILE)).ok();
        let status = match &archive {
            Some(store) => format!("Pronto. Archivio: {ARCHIVE_FILE} ({} analisi).", store.count()),
            None => format!("Pronto, ma non riesco ad aprire l'archivio {ARCHIVE_FILE}."),
        };
        let setting = |key: &str| archive.as_ref().and_then(|store| store.get_setting(key));
        let provider = match setting("provider").as_deref() {
            Some("chatgpt") => ProviderChoice::ChatGpt,
            _ => ProviderChoice::Ollama,
        };
        let model = setting("ollama_model").unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let openai_model = setting("openai_model").unwrap_or_else(|| DEFAULT_OPENAI_MODEL.to_string());
        let api_key = setting("openai_key").unwrap_or_default();
        let (control, snapshot) = control::spawn(CONTROL_PORT, cc.egui_ctx.clone());
        Self {
            photos: Vec::new(),
            selected: None,
            reader: None,
            analyzer: None,
            status,
            model,
            provider,
            openai_model,
            api_key,
            module: Module::Library,
            thumb_size: 150.0,
            folder: None,
            archive,
            reports: HashMap::new(),
            analyzing: false,
            queue: Vec::new(),
            queue_total: 0,
            queue_done: 0,
            selected_tag: None,
            control: Some(control),
            snapshot: Some(snapshot),
            ollama_models: Vec::new(),
            models_receiver: None,
            models_requested: false,
            view: ViewKind::Grid,
            search: String::new(),
            preview: None,
            preview_key: None,
        }
    }

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
                Msg::Analyzed(outcome) => {
                    self.store_outcome(*outcome);
                    self.analyzing = false;
                    self.queue_done += 1;
                }
                Msg::AnalyzedError(message) => {
                    self.status = message;
                    self.analyzing = false;
                    self.queue_done += 1;
                }
            }
        }
        if reader_done {
            self.reader = None;
        }
        if analyzer_done {
            self.analyzer = None;
        }
        let mut models_received: Option<Vec<String>> = None;
        let mut models_ended = false;
        if let Some(receiver) = &self.models_receiver {
            match receiver.try_recv() {
                Ok(models) => models_received = Some(models),
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => models_ended = true,
            }
        }
        if let Some(models) = models_received {
            self.ollama_models = models;
            self.models_receiver = None;
        } else if models_ended {
            self.models_receiver = None;
        }
        if self.reader.is_some() || self.analyzer.is_some() || self.models_receiver.is_some() {
            ctx.request_repaint();
        }
        // Se il batch non è occupato, passa alla prossima foto in coda.
        if !self.analyzing && !self.queue.is_empty() {
            self.pump_queue(ctx);
        }
    }

    fn store_outcome(&mut self, outcome: AnalysisOutcome) {
        if let Some(store) = &mut self.archive {
            let mut record = AnalysisRecord::new(outcome.file_name.clone(), outcome.source_key.clone());
            record.score = outcome.report.score;
            record.tags = outcome.report.tag.clone();
            record.model = Some(outcome.model.clone());
            record.report_json = outcome.report_json.clone();
            record.camera = outcome.camera.clone();
            record.iso = outcome.iso;
            if let Err(error) = store.save(record) {
                self.status = format!("{error}");
            }
        }
        self.reports.insert(
            outcome.source_key.clone(),
            AnalysisReport {
                score: outcome.report.score,
                tecnica: outcome.report.tecnica,
                composizione: outcome.report.composizione,
                messaggio: outcome.report.messaggio,
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

    /// Avvia l'analisi di un file, in un thread separato.
    fn analyze_path(&mut self, ctx: &egui::Context, path: PathBuf, name: String) {
        let provider = self.provider;
        let (model, key) = match provider {
            ProviderChoice::Ollama => (self.model.clone(), String::new()),
            ProviderChoice::ChatGpt => (self.openai_model.clone(), self.api_key.clone()),
        };
        let (sender, receiver) = mpsc::channel();
        self.analyzer = Some(receiver);
        self.analyzing = true;
        self.status = format!("Analizzo {name} con {model}… (può richiedere qualche minuto)");
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let outcome = match provider {
                ProviderChoice::Ollama => run_analysis(&OllamaCritic::new(model), &path),
                ProviderChoice::ChatGpt => run_analysis(&OpenAiCritic::new(model, key), &path),
            };
            match outcome {
                Ok(outcome) => {
                    let _ = sender.send(Msg::Analyzed(Box::new(outcome)));
                }
                Err(error) => {
                    let _ = sender.send(Msg::AnalyzedError(format!("{name}: {error}")));
                }
            }
            ctx.request_repaint();
        });
    }

    /// Analizza la foto scelta.
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
        self.analyze_path(ctx, path, name);
    }

    /// Mette in coda tutte le foto non ancora analizzate.
    fn start_batch(&mut self, ctx: &egui::Context) {
        if self.analyzing {
            return;
        }
        let mut pending = Vec::new();
        for photo in &self.photos {
            if let Some(path) = &photo.path {
                let key = path.to_string_lossy().to_string();
                if !self.reports.contains_key(&key) {
                    pending.push(path.clone());
                }
            }
        }
        self.queue = pending;
        self.queue_total = self.queue.len();
        self.queue_done = 0;
        if self.queue.is_empty() {
            self.status = "Nessuna foto da analizzare.".to_string();
            return;
        }
        self.status = format!("Analizzo {} foto…", self.queue_total);
        self.pump_queue(ctx);
    }

    /// Avvia l'analisi della prossima foto in coda (una per volta).
    fn pump_queue(&mut self, ctx: &egui::Context) {
        if self.analyzing || self.queue.is_empty() {
            return;
        }
        let path = self.queue.remove(0);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        self.analyze_path(ctx, path, name);
    }

    /// Imposta la valutazione a stelle della foto scelta.
    fn set_rating(&mut self, rating: i32) {
        let Some(photo) = self.selected.and_then(|index| self.photos.get(index)) else {
            return;
        };
        let Some(path) = photo.path.clone() else { return };
        let key = path.to_string_lossy().to_string();
        let name = photo.name.clone();
        if let Some(store) = &mut self.archive {
            let mut record = store
                .find(&key)
                .ok()
                .flatten()
                .unwrap_or_else(|| AnalysisRecord::new(name, key));
            record.rating = Some(rating);
            if let Err(error) = store.save(record) {
                self.status = format!("{error}");
            } else {
                self.status = format!("Valutazione: {rating} stelle.");
            }
        }
    }

    /// Rimuove la foto scelta dall'archivio.
    fn remove_from_archive(&mut self) {
        let Some(key) = self.selected_key() else {
            return;
        };
        if let Some(store) = &self.archive {
            if let Err(error) = store.delete(&key) {
                self.status = format!("{error}");
                return;
            }
        }
        self.reports.remove(&key);
        self.status = "Rimosso dall'archivio.".to_string();
    }

    /// Esporta il report della foto scelta in PDF.
    fn export_pdf(&mut self) {
        let Some(key) = self.selected_key() else {
            self.status = "Seleziona una foto con un'analisi.".to_string();
            return;
        };
        let Some(report) = self.reports.get(&key) else {
            self.status = "Nessuna analisi da esportare.".to_string();
            return;
        };
        let name = self
            .photos
            .get(self.selected.unwrap_or(0))
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "report".to_string());
        let mut lines = Vec::new();
        lines.push(format!(
            "Punteggio: {}",
            report
                .score
                .map(|s| s.to_string())
                .unwrap_or_else(|| "—".to_string())
        ));
        lines.push(format!("Modello: {}", report.model));
        if let Some(summary) = &report.summary {
            lines.push(format!("Sintesi: {summary}"));
        }
        if !report.tags.is_empty() {
            lines.push(format!("Tag: {}", report.tags.join(", ")));
        }
        lines.push(String::new());
        lines.push("Punti di forza:".to_string());
        for item in &report.strengths {
            lines.push(format!("- {item}"));
        }
        lines.push(String::new());
        lines.push("Punti di debolezza:".to_string());
        for item in &report.weaknesses {
            lines.push(format!("- {item}"));
        }
        let pdf = postcritica_engine::report_pdf(&name, &lines);
        let default_name = format!("{name}.pdf");
        if let Some(path) = rfd::FileDialog::new().set_file_name(default_name).save_file() {
            match std::fs::write(&path, &pdf) {
                Ok(()) => self.status = format!("PDF scritto in {}", path.display()),
                Err(error) => self.status = format!("Non scrivo il PDF: {error}"),
            }
        }
    }

    /// Importa le analisi dal database del programma Python.
    fn import_python(&mut self) {
        let Some(path) = rfd::FileDialog::new().add_filter("SQLite", &["db"]).pick_file() else {
            return;
        };
        let Some(store) = &mut self.archive else {
            self.status = "Archivio non disponibile.".to_string();
            return;
        };
        match store.import_python(&path) {
            Ok(count) => self.status = format!("Importate {count} analisi da {}.", path.display()),
            Err(error) => self.status = format!("Import non riuscito: {error}"),
        }
    }

    /// Legge i comandi dal canale di controllo e aggiorna la fotografia dello stato.
    fn handle_control(&mut self, ctx: &egui::Context) {
        let mut commands = Vec::new();
        if let Some(receiver) = &self.control {
            while let Ok(command) = receiver.try_recv() {
                commands.push(command);
            }
        }
        for command in commands {
            match command {
                control::Command::OpenFolder(path) => self.start_folder(ctx, PathBuf::from(path)),
                control::Command::AnalyzeAll => self.start_batch(ctx),
                control::Command::Select(index) => self.selected = Some(index),
                control::Command::SetModule(name) => {
                    if let Some(module) = Module::ALL
                        .into_iter()
                        .find(|module| module.label().eq_ignore_ascii_case(&name))
                    {
                        self.module = module;
                    }
                }
                control::Command::Demo => self.load_demo(ctx),
                control::Command::SetView(view) => {
                    self.view = if view.eq_ignore_ascii_case("loupe") || view.eq_ignore_ascii_case("foto") {
                        ViewKind::Loupe
                    } else {
                        ViewKind::Grid
                    };
                }
                control::Command::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            }
        }
        if let Some(snapshot) = &self.snapshot {
            if let Ok(mut snap) = snapshot.lock() {
                snap.photos = self.photos.len();
                snap.analyzed = self.reports.len();
                snap.archive = self.archive.as_ref().map_or(0, |store| store.count());
                snap.status = self.status.clone();
                snap.module = self.module.label().to_string();
            }
        }
    }

    /// Esporta le analisi dell'archivio in un file (CSV, JSON o JSONL).
    fn export_records(&mut self, format: ExportFormat) {
        let records = self
            .archive
            .as_ref()
            .and_then(|store| store.history().ok())
            .unwrap_or_default();
        if records.is_empty() {
            self.status = "Nessuna analisi da esportare.".to_string();
            return;
        }
        let refs: Vec<&AnalysisRecord> = records.iter().collect();
        let (contents, name) = match format {
            ExportFormat::Csv => (postcritica_engine::to_csv(&refs), "postcritica.csv"),
            ExportFormat::Json => (postcritica_engine::to_json(&refs), "postcritica.json"),
            ExportFormat::JsonL => (postcritica_engine::to_jsonl(&refs), "postcritica.jsonl"),
        };
        if let Some(path) = rfd::FileDialog::new().set_file_name(name).save_file() {
            match std::fs::write(&path, contents) {
                Ok(()) => self.status = format!("Scritto {}.", path.display()),
                Err(error) => self.status = format!("Non scrivo: {error}"),
            }
        }
    }

    /// Chiede una volta, all'avvio, l'elenco dei modelli Ollama.
    fn ensure_models(&mut self, ctx: &egui::Context) {
        if !self.models_requested {
            self.models_requested = true;
            self.refresh_models(ctx);
        }
    }

    /// Rilegge l'elenco dei modelli Ollama installati, in un thread.
    fn refresh_models(&mut self, ctx: &egui::Context) {
        let (sender, receiver) = mpsc::channel();
        self.models_receiver = Some(receiver);
        self.status = "Leggo i modelli di Ollama…".to_string();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let models = OllamaCritic::available_models(OLLAMA_URL).unwrap_or_default();
            let _ = sender.send(models);
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

    fn library(&mut self, ui: &mut egui::Ui) {
        let query = self.search.trim().to_lowercase();
        let visible: Vec<usize> = self
            .photos
            .iter()
            .enumerate()
            .filter(|(_, photo)| query.is_empty() || photo.name.to_lowercase().contains(&query))
            .map(|(index, _)| index)
            .collect();
        let analyzed = visible
            .iter()
            .filter(|index| {
                self.photos
                    .get(**index)
                    .and_then(|photo| photo.path.as_ref())
                    .is_some_and(|path| self.reports.contains_key(&path.to_string_lossy().into_owned()))
            })
            .count();

        if self.view == ViewKind::Loupe {
            match self.selected.and_then(|index| self.photos.get(index)) {
                Some(photo) => {
                    let (width, height) = photo.inspection.original;
                    ui.horizontal(|ui| {
                        ui.strong(&photo.name);
                        if let Some(day) = &photo.inspection.metadata.shot_at {
                            ui.weak(day);
                        }
                        ui.weak(format!("· {width} × {height}"));
                    });
                }
                None => {
                    ui.strong("Nessuna foto scelta");
                }
            }
        } else {
            ui.horizontal(|ui| {
                ui.heading("Tutte le foto");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.weak(format!("{} foto · {analyzed} analizzate", visible.len()));
                });
            });
        }
        ui.separator();

        if visible.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(if self.photos.is_empty() {
                    "Nessuna foto. Apri una cartella dal menu «File»."
                } else {
                    "Nessuna foto corrisponde alla ricerca."
                });
            });
            return;
        }

        if self.view == ViewKind::Loupe {
            self.loupe(ui);
            return;
        }

        let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
        for &index in &visible {
            if let Some(photo) = self.photos.get(index) {
                let day = photo_day(photo);
                match groups.iter_mut().find(|(label, _)| *label == day) {
                    Some((_, list)) => list.push(index),
                    None => groups.push((day, vec![index])),
                }
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

    /// La foto scelta, grande, con l'anteprima adattata allo spazio.
    fn loupe(&self, ui: &mut egui::Ui) {
        let Some(photo) = self.selected.and_then(|index| self.photos.get(index)) else {
            ui.centered_and_justified(|ui| {
                ui.label("Scegli una foto dalla striscia qui sotto.");
            });
            return;
        };
        let available = ui.available_size();
        let (original_width, original_height) = photo.inspection.original;
        let aspect = if original_width > 0 && original_height > 0 {
            original_width as f32 / original_height as f32
        } else {
            photo.inspection.thumb.width() as f32 / photo.inspection.thumb.height().max(1) as f32
        };
        let mut size = egui::vec2(available.x, available.x / aspect.max(0.01));
        if size.y > available.y {
            size = egui::vec2(available.y * aspect, available.y);
        }
        let texture = self.preview.as_ref().unwrap_or(&photo.thumb);
        ui.centered_and_justified(|ui| {
            ui.add(egui::Image::new(texture).fit_to_exact_size(size));
        });
    }

    /// La striscia di miniature in basso, da cui scegliere la foto.
    fn filmstrip(&mut self, ui: &mut egui::Ui) {
        let query = self.search.trim().to_lowercase();
        let visible: Vec<usize> = self
            .photos
            .iter()
            .enumerate()
            .filter(|(_, photo)| query.is_empty() || photo.name.to_lowercase().contains(&query))
            .map(|(index, _)| index)
            .collect();
        if visible.is_empty() {
            return;
        }
        let tokens = Tokens::get(ui.ctx());
        let mut clicked = None;
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                for &index in &visible {
                    if let Some(photo) = self.photos.get(index) {
                        let image = egui::Image::new(&photo.thumb)
                            .fit_to_exact_size(egui::vec2(88.0, 62.0))
                            .sense(egui::Sense::click());
                        let response = ui.add(image).on_hover_text(&photo.name);
                        if self.selected == Some(index) {
                            ui.painter().rect_stroke(
                                response.rect,
                                2.0,
                                egui::Stroke::new(2.5, tokens.accent),
                                egui::StrokeKind::Inside,
                            );
                        }
                        if response.clicked() {
                            clicked = Some(index);
                        }
                    }
                }
            });
        });
        if let Some(index) = clicked {
            self.selected = Some(index);
        }
    }

    /// Prepara l'anteprima grande della foto scelta quando si usa la vista a foto singola.
    fn ensure_preview(&mut self, ctx: &egui::Context) {
        if self.view != ViewKind::Loupe {
            return;
        }
        let key = self.selected_key();
        if key == self.preview_key {
            return;
        }
        self.preview_key = key.clone();
        self.preview = None;
        let Some(path) = key.map(PathBuf::from) else {
            return;
        };
        if let Ok(image) = postcritica_raster::decode_thumbnail(&path, 1400) {
            self.preview = Some(make_texture(ctx, &format!("preview-{}", path.display()), &image));
        }
    }

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
            ui.label("Nessuna analisi salvata. Apri una foto e premi «Analizza con Ollama».");
            return;
        }
        let tokens = Tokens::get(ui.ctx());
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
                            .color(tokens.score_colour(record.score)),
                    );
                    ui.label(&record.file_name);
                    if let Some(camera) = &record.camera {
                        ui.weak(camera);
                    }
                });
                ui.weak(record.tags.join(" · "));
                ui.separator();
            }
        });
    }

    /// Le statistiche fotografiche, con i grafici (come il programma Python).
    fn stats_view(&self, ui: &mut egui::Ui) {
        let records = self
            .archive
            .as_ref()
            .and_then(|store| store.history().ok())
            .unwrap_or_default();
        ui.horizontal(|ui| {
            ui.heading("Statistiche fotografiche");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.weak(format!("{} foto analizzate", records.len()));
            });
        });
        ui.separator();
        if records.is_empty() {
            ui.label("Nessuna analisi. Analizza qualche foto per vedere le statistiche.");
            return;
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            let tokens = Tokens::get(ui.ctx());
            ui.add_space(6.0);
            ui.strong("Distribuzione dei punteggi");
            bar_chart(ui, &score_buckets(&records), tokens.accent);

            ui.add_space(14.0);
            ui.strong("Fotocamere più usate");
            bar_chart(
                ui,
                &top_values(records.iter().filter_map(|r| r.camera.clone()), 8),
                tokens.accent,
            );

            ui.add_space(14.0);
            ui.strong("Distribuzione ISO");
            bar_chart(ui, &iso_buckets(&records), tokens.accent);

            ui.add_space(14.0);
            ui.strong("Generi e tag più frequenti");
            bar_chart(
                ui,
                &top_values(records.iter().flat_map(|r| r.tags.clone()), 12),
                tokens.accent,
            );
        });
    }

    /// Le raccolte per tag, ricavate dalle analisi salvate.
    fn collections_view(&mut self, ui: &mut egui::Ui) {
        let records = self
            .archive
            .as_ref()
            .and_then(|store| store.history().ok())
            .unwrap_or_default();
        let mut by_tag: HashMap<String, Vec<usize>> = HashMap::new();
        for (index, record) in records.iter().enumerate() {
            for tag in &record.tags {
                by_tag.entry(tag.clone()).or_default().push(index);
            }
        }
        ui.horizontal(|ui| {
            ui.heading("Raccolte per tag");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.weak(format!("{} tag", by_tag.len()));
            });
        });
        ui.separator();
        if by_tag.is_empty() {
            ui.label("Ancora nessun tag: analizza qualche foto.");
            return;
        }
        let mut tags: Vec<(String, usize)> = by_tag
            .iter()
            .map(|(tag, list)| (tag.clone(), list.len()))
            .collect();
        tags.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        ui.horizontal_wrapped(|ui| {
            for (tag, count) in &tags {
                let selected = self.selected_tag.as_deref() == Some(tag.as_str());
                if ui
                    .selectable_label(selected, format!("{tag} ({count})"))
                    .clicked()
                {
                    self.selected_tag = Some(tag.clone());
                }
            }
        });
        ui.add_space(8.0);
        let tokens = Tokens::get(ui.ctx());
        match self
            .selected_tag
            .clone()
            .and_then(|tag| by_tag.get(&tag).map(|list| (tag, list.clone())))
        {
            Some((tag, indices)) => {
                ui.strong(format!("Foto con «{tag}»: {}", indices.len()));
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for index in indices {
                        if let Some(record) = records.get(index) {
                            ui.horizontal(|ui| {
                                let score = record
                                    .score
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| "—".to_string());
                                ui.label(
                                    egui::RichText::new(format!("{score:>3}"))
                                        .strong()
                                        .color(tokens.score_colour(record.score)),
                                );
                                ui.label(&record.file_name);
                            });
                        }
                    }
                });
            }
            None => {
                ui.weak("Scegli un tag per vedere la raccolta.");
            }
        }
    }

    /// Salva le impostazioni nell'archivio.
    fn persist(&self) {
        if let Some(store) = &self.archive {
            let _ = store.set_setting("provider", self.provider.key());
            let _ = store.set_setting("ollama_model", &self.model);
            let _ = store.set_setting("openai_model", &self.openai_model);
            let _ = store.set_setting("openai_key", &self.api_key);
        }
    }

    /// Il modulo Setup: provider, modelli e chiave.
    fn setup_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Setup");
        ui.separator();
        ui.strong("Provider di analisi");
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.provider == ProviderChoice::Ollama, "Ollama (locale)")
                .clicked()
            {
                self.provider = ProviderChoice::Ollama;
                self.persist();
            }
            if ui
                .selectable_label(self.provider == ProviderChoice::ChatGpt, "ChatGPT (OpenAI)")
                .clicked()
            {
                self.provider = ProviderChoice::ChatGpt;
                self.persist();
            }
        });
        ui.add_space(10.0);
        ui.strong("Ollama");
        let models = self.ollama_models.clone();
        let mut chosen: Option<String> = None;
        ui.horizontal(|ui| {
            ui.label("Modello:");
            egui::ComboBox::from_id_salt("ollama-model")
                .selected_text(self.model.clone())
                .width(260.0)
                .show_ui(ui, |ui| {
                    if models.is_empty() {
                        ui.weak("nessun modello trovato (Ollama è avviato?)");
                    }
                    for name in &models {
                        if ui.selectable_label(self.model == *name, name).clicked() {
                            chosen = Some(name.clone());
                        }
                    }
                });
            if ui.button("Aggiorna elenco").clicked() {
                self.models_requested = false;
                self.refresh_models(ui.ctx());
            }
        });
        if let Some(name) = chosen {
            self.model = name;
            self.persist();
        }
        ui.add_space(10.0);
        ui.strong("ChatGPT (OpenAI)");
        ui.horizontal(|ui| {
            ui.label("Modello:");
            if ui
                .add(egui::TextEdit::singleline(&mut self.openai_model).desired_width(180.0))
                .lost_focus()
            {
                self.persist();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Chiave API:");
            if ui
                .add(
                    egui::TextEdit::singleline(&mut self.api_key)
                        .password(true)
                        .desired_width(260.0),
                )
                .lost_focus()
            {
                self.persist();
            }
        });
        ui.add_space(8.0);
        if ui.button("Salva impostazioni").clicked() {
            self.persist();
            self.status = "Impostazioni salvate.".to_string();
        }
        ui.add_space(10.0);
        ui.weak("La chiave resta nell'archivio locale (postcritica.db), non viene mostrata altrove.");
    }

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
        {
            let tokens = Tokens::get(ui.ctx());
            let current = photo
                .path
                .as_ref()
                .and_then(|path| {
                    self.archive
                        .as_ref()
                        .and_then(|store| store.find(&path.to_string_lossy()).ok().flatten())
                })
                .and_then(|record| record.rating)
                .unwrap_or(0);
            ui.horizontal(|ui| {
                ui.label("Valutazione:");
                for star in 1..=5 {
                    let filled = star <= current;
                    let (text, colour) = if filled {
                        ("★", tokens.star)
                    } else {
                        ("☆", tokens.text_disabled)
                    };
                    if ui
                        .add(egui::Button::new(egui::RichText::new(text).color(colour)).frame(false))
                        .clicked()
                    {
                        ui.ctx()
                            .data_mut(|data| data.insert_temp(egui::Id::new("rating"), star));
                    }
                }
                if ui.button("Esporta PDF").clicked() {
                    ui.ctx()
                        .data_mut(|data| data.insert_temp(egui::Id::new("pdf"), true));
                }
            });
        }
        ui.add_space(6.0);
        ui.heading(&photo.name);
        let (width, height) = photo.inspection.original;
        ui.label(format!("{width} × {height} px"));
        if let Some(day) = photo.inspection.metadata.shot_at.clone() {
            ui.weak(day);
        }
        ui.add_space(8.0);
        histogram(ui, &photo.inspection.thumb);
        ui.add_space(8.0);
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

        let tokens = Tokens::get(ui.ctx());
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
                        .color(tokens.score_colour(report.score)),
                );
                let part =
                    |value: Option<i32>| value.map(|v| v.to_string()).unwrap_or_else(|| "—".to_string());
                ui.weak(format!(
                    "Tecnica {} · Composizione {} · Messaggio {}",
                    part(report.tecnica),
                    part(report.composizione),
                    part(report.messaggio),
                ));
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
                list(ui, "Punti di forza", &report.strengths, true, tokens);
                list(ui, "Punti di debolezza", &report.weaknesses, false, tokens);
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
        let tokens = Tokens::get(&ctx);
        self.drain(&ctx);
        self.handle_control(&ctx);
        self.ensure_models(&ctx);
        self.ensure_preview(&ctx);

        let requested = ctx
            .data_mut(|data| data.remove_temp::<bool>(egui::Id::new("analizza")))
            .unwrap_or(false);
        if requested {
            self.start_analysis(&ctx);
        }
        if let Some(rating) = ctx.data_mut(|data| data.remove_temp::<i32>(egui::Id::new("rating"))) {
            self.set_rating(rating);
        }
        if ctx
            .data_mut(|data| data.remove_temp::<bool>(egui::Id::new("pdf")))
            .unwrap_or(false)
        {
            self.export_pdf();
        }

        egui::Panel::top("menu").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Apri cartella…").clicked() {
                        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                            self.start_folder(&ctx, dir);
                        }
                        ui.close();
                    }
                    if ui.button("Analizza tutte").clicked() {
                        self.start_batch(&ctx);
                        ui.close();
                    }
                    if ui.button("Importa dal programma Python…").clicked() {
                        self.import_python();
                        ui.close();
                    }
                    ui.menu_button("Esporta archivio", |ui| {
                        if ui.button("CSV").clicked() {
                            self.export_records(ExportFormat::Csv);
                            ui.close();
                        }
                        if ui.button("JSON").clicked() {
                            self.export_records(ExportFormat::Json);
                            ui.close();
                        }
                        if ui.button("JSONL").clicked() {
                            self.export_records(ExportFormat::JsonL);
                            ui.close();
                        }
                    });
                    ui.separator();
                    if ui.button("Esci").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Modifica", |ui| {
                    if ui
                        .add_enabled(self.selected.is_some(), egui::Button::new("Deseleziona"))
                        .clicked()
                    {
                        self.selected = None;
                        ui.close();
                    }
                    ui.menu_button("Valutazione", |ui| {
                        for stars in 0..=5 {
                            let label = if stars == 0 {
                                "Nessuna (0)".to_string()
                            } else {
                                "★".repeat(stars)
                            };
                            if ui.button(label).clicked() {
                                self.set_rating(stars as i32);
                                ui.close();
                            }
                        }
                    });
                    if ui
                        .add_enabled(
                            self.selected.is_some(),
                            egui::Button::new("Rimuovi dall'archivio"),
                        )
                        .clicked()
                    {
                        self.remove_from_archive();
                        ui.close();
                    }
                });
                ui.menu_button("Vista", |ui| {
                    for module in Module::ALL {
                        if ui
                            .selectable_label(self.module == module, module.label())
                            .clicked()
                        {
                            self.module = module;
                            ui.close();
                        }
                    }
                    ui.separator();
                    if ui.button("Miniature più grandi").clicked() {
                        self.thumb_size = (self.thumb_size + 20.0).min(280.0);
                        ui.close();
                    }
                    if ui.button("Miniature più piccole").clicked() {
                        self.thumb_size = (self.thumb_size - 20.0).max(90.0);
                        ui.close();
                    }
                });
                ui.menu_button("Foto", |ui| {
                    if ui
                        .add_enabled(self.selected.is_some(), egui::Button::new("Analizza"))
                        .clicked()
                    {
                        self.start_analysis(&ctx);
                        ui.close();
                    }
                    if ui.button("Analizza tutte").clicked() {
                        self.start_batch(&ctx);
                        ui.close();
                    }
                    if ui
                        .add_enabled(self.selected.is_some(), egui::Button::new("Esporta PDF"))
                        .clicked()
                    {
                        self.export_pdf();
                        ui.close();
                    }
                });
                ui.separator();
                ui.separator();
                ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .hint_text("Cerca per nome…")
                        .desired_width(280.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.weak("Ollama:");
                    ui.add(egui::TextEdit::singleline(&mut self.model).desired_width(150.0));
                });
            });
        });

        egui::Panel::bottom("barra")
            .size_range(egui::Rangef::new(30.0, 46.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.view == ViewKind::Grid, "▦")
                        .on_hover_text("Griglia")
                        .clicked()
                    {
                        self.view = ViewKind::Grid;
                    }
                    if ui
                        .selectable_label(self.view == ViewKind::Loupe, "▣")
                        .on_hover_text("Foto singola")
                        .clicked()
                    {
                        self.view = ViewKind::Loupe;
                    }
                    ui.separator();
                    if ui
                        .add_enabled(self.selected.is_some(), egui::Button::new("Analizza"))
                        .clicked()
                    {
                        self.start_analysis(&ctx);
                    }
                    ui.separator();
                    for _ in 0..5 {
                        ui.label(egui::RichText::new("★").color(tokens.text_disabled));
                    }
                    if self.queue_total > 0 {
                        ui.separator();
                        let fraction = self.queue_done as f32 / self.queue_total as f32;
                        ui.add(
                            egui::ProgressBar::new(fraction)
                                .desired_width(140.0)
                                .text(format!("{}/{}", self.queue_done, self.queue_total)),
                        );
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.weak(&self.status);
                        ui.separator();
                        ui.add(egui::Slider::new(&mut self.thumb_size, 90.0..=280.0).show_value(false));
                        ui.weak("Miniature");
                    });
                });
            });

        if !self.photos.is_empty() {
            egui::Panel::bottom("striscia")
                .size_range(egui::Rangef::new(72.0, 118.0))
                .show(ui, |ui| {
                    ui.add_space(2.0);
                    self.filmstrip(ui);
                });
        }

        egui::Panel::left("libreria")
            .default_size(tokens.panel_w)
            .size_range(egui::Rangef::new(210.0, 380.0))
            .show(ui, |ui| {
                ui.add_space(4.0);
                let _ = ui.selectable_label(true, "Libreria");
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
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
                            if ui
                                .add_enabled(!self.photos.is_empty(), egui::Button::new("Analizza tutte"))
                                .clicked()
                            {
                                self.start_batch(&ctx);
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

        egui::Panel::right("moduli")
            .exact_size(tokens.strip_w)
            .show(ui, |ui| {
                ui.add_space(6.0);
                for module in Module::ALL {
                    if module_button(ui, self.module == module, module, tokens).clicked() {
                        self.module = module;
                    }
                    ui.add_space(4.0);
                }
            });

        egui::Panel::right("pannello")
            .default_size(400.0)
            .size_range(egui::Rangef::new(340.0, 560.0))
            .show(ui, |ui| {
                ui.add_space(4.0);
                let _ = ui.selectable_label(true, "Strumenti");
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.inspector(ui);
                });
            });

        let frame = egui::Frame::default()
            .fill(tokens.grid_bg)
            .inner_margin(egui::Margin::same(8));
        egui::CentralPanel::default()
            .frame(frame)
            .show(ui, |ui| match self.module {
                Module::Library | Module::Analysis => self.library(ui),
                Module::Archive => self.archive_view(ui),
                Module::Stats => self.stats_view(ui),
                Module::Collections => self.collections_view(ui),
                Module::Setup => self.setup_view(ui),
            });
    }
}

/// Un istogramma RGB dell'anteprima.
fn histogram(ui: &mut egui::Ui, image: &Rgba8) {
    let tokens = Tokens::get(ui.ctx());
    let width = ui.available_width().clamp(160.0, 380.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 96.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 3.0, tokens.inset);

    let mut red = [0u32; 256];
    let mut green = [0u32; 256];
    let mut blue = [0u32; 256];
    for pixel in image.pixels() {
        red[pixel[0] as usize] += 1;
        green[pixel[1] as usize] += 1;
        blue[pixel[2] as usize] += 1;
    }
    let max = red
        .iter()
        .chain(&green)
        .chain(&blue)
        .copied()
        .max()
        .unwrap_or(1)
        .max(1) as f32;

    let painter = ui.painter_at(rect);
    let bar_width = rect.width() / 256.0;
    for (bins, colour) in [
        (&red, tokens.reject),
        (&green, tokens.positive),
        (&blue, tokens.accent),
    ] {
        for (index, value) in bins.iter().enumerate() {
            if *value == 0 {
                continue;
            }
            let height = rect.height() * (*value as f32 / max);
            let x = rect.min.x + index as f32 * bar_width;
            let bar = egui::Rect::from_min_max(
                egui::pos2(x, rect.max.y - height),
                egui::pos2(x + bar_width + 0.5, rect.max.y),
            );
            painter.rect_filled(bar, 0.0, colour.gamma_multiply(0.55));
        }
    }
}

/// Un grafico a barre orizzontali con etichette e conteggi.
fn bar_chart(ui: &mut egui::Ui, data: &[(String, usize)], colour: egui::Color32) {
    if data.is_empty() {
        ui.weak("(nessun dato)");
        return;
    }
    let max = data.iter().map(|(_, value)| *value).max().unwrap_or(1).max(1);
    let tokens = Tokens::get(ui.ctx());
    for (label, value) in data {
        ui.horizontal(|ui| {
            ui.add_sized(
                [190.0, 18.0],
                egui::Label::new(egui::RichText::new(label).color(tokens.text_label)),
            );
            let width = (*value as f32 / max as f32) * 260.0;
            let (rect, _) = ui.allocate_exact_size(egui::vec2(262.0, 16.0), egui::Sense::hover());
            let bar = egui::Rect::from_min_size(rect.min, egui::vec2(width.max(2.0), rect.height()));
            ui.painter().rect_filled(bar, 2.0, colour);
            ui.weak(value.to_string());
        });
    }
}

/// I punteggi in dieci fasce.
fn score_buckets(records: &[AnalysisRecord]) -> Vec<(String, usize)> {
    let mut buckets: Vec<(String, usize)> = (0..10)
        .map(|i| (format!("{}–{}", i * 10, i * 10 + 9), 0))
        .collect();
    for record in records {
        if let Some(score) = record.score {
            let index = (score.clamp(0, 100) / 10).min(9) as usize;
            buckets[index].1 += 1;
        }
    }
    buckets
}

/// Gli ISO in quattro fasce.
fn iso_buckets(records: &[AnalysisRecord]) -> Vec<(String, usize)> {
    let ranges = [
        ("fino a 200", 0u32, 200u32),
        ("201–800", 201, 800),
        ("801–3200", 801, 3200),
        ("oltre 3200", 3201, u32::MAX),
    ];
    let mut buckets: Vec<(String, usize)> = ranges
        .iter()
        .map(|(label, _, _)| (label.to_string(), 0))
        .collect();
    for record in records {
        if let Some(iso) = record.iso {
            for (index, (_, low, high)) in ranges.iter().enumerate() {
                if iso >= *low && iso <= *high {
                    buckets[index].1 += 1;
                    break;
                }
            }
        }
    }
    buckets
}

/// I valori più frequenti di una serie, in ordine decrescente.
fn top_values(values: impl Iterator<Item = String>, limit: usize) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for value in values {
        let trimmed = value.trim().to_string();
        if !trimmed.is_empty() {
            *counts.entry(trimmed).or_insert(0) += 1;
        }
    }
    let mut list: Vec<(String, usize)> = counts.into_iter().collect();
    list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    list.truncate(limit);
    list
}

/// Un pulsante quadrato della striscia dei moduli, con l'icona disegnata in codice (lavoro originale).
fn module_button(ui: &mut egui::Ui, selected: bool, module: Module, tokens: Tokens) -> egui::Response {
    let size = egui::vec2(34.0, 34.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let (bg, colour) = if selected {
        (tokens.accent.gamma_multiply(0.8), tokens.text)
    } else if response.hovered() {
        (tokens.hover, tokens.thumb_hover)
    } else {
        (egui::Color32::TRANSPARENT, tokens.icon)
    };
    if bg != egui::Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, 4.0, bg);
    }
    draw_module_icon(ui.painter(), rect.shrink(9.0), colour, module);
    response.on_hover_text(module.label())
}

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
        Module::Setup => {
            for i in 0..2 {
                let y = rect.min.y + (i as f32 + 0.5) * rect.height() / 2.0;
                painter.line_segment([egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)], stroke);
                let knob = rect.min.x + rect.width() * if i == 0 { 0.35 } else { 0.65 };
                painter.circle_filled(egui::pos2(knob, y), 2.0, colour);
            }
        }
    }
}

fn list(ui: &mut egui::Ui, title: &str, items: &[String], positive: bool, tokens: Tokens) {
    if items.is_empty() {
        return;
    }
    ui.add_space(4.0);
    ui.label(egui::RichText::new(title).strong());
    let colour = if positive { tokens.positive } else { tokens.reject };
    for item in items {
        ui.label(egui::RichText::new(format!("• {item}")).color(colour));
    }
}

fn photo_day(photo: &Photo) -> String {
    match photo.inspection.metadata.shot_at.as_deref() {
        Some(value) if value.chars().count() >= 10 => value.chars().take(10).collect(),
        Some(value) if !value.trim().is_empty() => value.to_string(),
        _ => "Senza data".to_string(),
    }
}

fn thumbnail(ui: &mut egui::Ui, photo: &Photo, selected: bool, size: f32) -> egui::Response {
    let tokens = Tokens::get(ui.ctx());
    let target = egui::vec2(size, size * 0.72);
    let image = egui::Image::new(&photo.thumb)
        .fit_to_exact_size(target)
        .sense(egui::Sense::click());
    let response = ui.add(image);
    if selected {
        ui.painter().rect_stroke(
            response.rect,
            3.0,
            egui::Stroke::new(2.5, tokens.accent),
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
