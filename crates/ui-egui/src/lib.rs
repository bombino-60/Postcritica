//! L'interfaccia desktop (egui/eframe) di Postcritica.
//!
//! Prima versione usabile: si apre una cartella di foto (o si carica una demo), si vedono le miniature
//! in una griglia e, selezionandone una, i metadati e il profilo visivo. Tutto in locale, senza rete.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};

use eframe::egui;
use postcritica_engine::{inspect_photo, rgba_bytes, Inspection, PhotoMetadata, Rgba8, VisualProfile};

/// Apre la finestra dell'app. Ritorna quando l'utente la chiude.
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_title("Postcritica"),
        ..Default::default()
    };
    eframe::run_native("Postcritica", options, Box::new(|_cc| Ok(Box::new(App::new()))))
}

/// Una foto mostrata nella griglia.
struct Photo {
    name: String,
    thumb: egui::TextureHandle,
    inspection: Inspection,
}

/// Un messaggio dal thread che legge la cartella.
enum Msg {
    Photo(Box<Photo>),
    Progress(usize, usize),
    Error(String),
    Done,
}

/// Lo stato dell'app.
struct App {
    photos: Vec<Photo>,
    selected: Option<usize>,
    receiver: Option<Receiver<Msg>>,
    status: String,
}

impl App {
    fn new() -> Self {
        Self {
            photos: Vec::new(),
            selected: None,
            receiver: None,
            status: "Pronto. Apri una cartella, oppure premi «Demo».".to_string(),
        }
    }

    /// Raccoglie i messaggi del thread di lettura e aggiorna lo stato.
    fn drain(&mut self, ctx: &egui::Context) {
        let mut messages = Vec::new();
        let mut done = false;
        if let Some(receiver) = &self.receiver {
            loop {
                match receiver.try_recv() {
                    Ok(message) => messages.push(message),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        done = true;
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
                    done = true;
                    self.status = format!("{} foto.", self.photos.len());
                }
            }
        }
        if done {
            self.receiver = None;
            self.status = format!("{} foto.", self.photos.len());
        }
        if self.receiver.is_some() {
            ctx.request_repaint();
        }
    }

    /// Avvia la lettura di una cartella in un thread separato, così la finestra non si blocca.
    fn start_folder(&mut self, ctx: &egui::Context, dir: PathBuf) {
        self.photos.clear();
        self.selected = None;
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        self.status = format!("Leggo {}…", dir.display());
        let ctx = ctx.clone();
        std::thread::spawn(move || read_folder(&dir, &ctx, &sender));
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
                    let r = (fx * 255.0) as u8;
                    let g = (fy * 255.0) as u8;
                    let b = (stripe * 255.0) as u8;
                    data.push([r, g, b, 255]);
                }
            }
            let Some(image) = Rgba8::new(width, height, data) else {
                continue;
            };
            let profile = VisualProfile::of(&image);
            let thumb = make_texture(ctx, &format!("demo-{index}"), &image);
            self.photos.push(Photo {
                name: format!("demo-{index:02}.png"),
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

    /// Il pannello di destra: metadati e profilo visivo della foto scelta.
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
        field(ui, "Rating", metadata.rating.map(|v| v.to_string()));
        ui.add_space(6.0);
        ui.separator();
        ui.strong("Profilo visivo");
        let profile = &photo.inspection.profile;
        ui.label(format!("Luminosità: {:.0}%", profile.brightness * 100.0));
        ui.label(format!("Contrasto: {:.0}%", profile.contrast * 100.0));
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new("L'analisi con Ollama o ChatGPT arriva più avanti.")
                .small()
                .weak(),
        );
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
                ui.label(&self.status);
            });
            ui.add_space(4.0);
        });

        egui::Panel::right("ispettore")
            .min_size(280.0)
            .default_size(320.0)
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

/// Disegna una miniatura cliccabile, con la cornice quando è quella scelta.
fn thumbnail(ui: &mut egui::Ui, photo: &Photo, selected: bool) -> egui::Response {
    let size = egui::vec2(132.0, 100.0);
    let image = egui::Image::new(&photo.thumb)
        .fit_to_exact_size(size)
        .sense(egui::Sense::click());
    let response = ui.add(image);
    if selected {
        ui.painter().rect_stroke(
            response.rect,
            3.0,
            egui::Stroke::new(2.5, egui::Color32::from_rgb(0, 150, 255)),
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
