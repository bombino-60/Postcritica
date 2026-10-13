//! Un piccolo canale di controllo locale (JSON-lines su TCP) per pilotare l'app.
//!
//! Come il canale di controllo del riferimento LightCraft: un agente può leggere lo stato e chiedere
//! azioni (aprire una cartella, analizzare tutto, selezionare, cambiare modulo, chiudere) senza toccare
//! mouse e tastiera. Si ascolta su `127.0.0.1:<porta>`.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use eframe::egui;

/// Un comando ricevuto dal canale di controllo.
pub enum Command {
    OpenFolder(String),
    AnalyzeAll,
    Select(usize),
    SetModule(String),
    /// Carica le immagini della demo (solo per le prove: non è nell'interfaccia).
    Demo,
    Quit,
}

/// La fotografia dello stato che il canale può leggere.
#[derive(Default, Clone)]
pub struct Snapshot {
    pub photos: usize,
    pub analyzed: usize,
    pub archive: usize,
    pub status: String,
    pub module: String,
}

/// Avvia il server su `127.0.0.1:porta`. Restituisce il ricevitore dei comandi e lo stato condiviso.
///
/// Il `Context` di egui serve a richiedere un ridisegno quando arriva un comando: senza, l'app
/// inattiva non elabora i comandi finché non riceve un evento.
pub fn spawn(port: u16, ctx: egui::Context) -> (Receiver<Command>, Arc<Mutex<Snapshot>>) {
    let (sender, receiver) = std::sync::mpsc::channel();
    let snapshot = Arc::new(Mutex::new(Snapshot::default()));
    let shared = Arc::clone(&snapshot);
    std::thread::spawn(move || {
        let Ok(listener) = TcpListener::bind(("127.0.0.1", port)) else {
            eprintln!("canale di controllo: porta {port} non disponibile");
            return;
        };
        for stream in listener.incoming().flatten() {
            let sender = sender.clone();
            let shared = Arc::clone(&shared);
            let ctx = ctx.clone();
            std::thread::spawn(move || serve(stream, sender, shared, ctx));
        }
    });
    (receiver, snapshot)
}

fn serve(stream: TcpStream, sender: Sender<Command>, snapshot: Arc<Mutex<Snapshot>>, ctx: egui::Context) {
    let Ok(reader_stream) = stream.try_clone() else {
        return;
    };
    let reader = BufReader::new(reader_stream);
    let mut writer = stream;
    for line in reader.lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => {
                let _ = writeln!(writer, "{{\"ok\":false,\"error\":\"json non valido\"}}");
                continue;
            }
        };
        let method = value.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = value.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let params = value.get("params").cloned().unwrap_or(serde_json::Value::Null);
        let reply = match method {
            "status" => {
                let snap = snapshot.lock().map(|s| s.clone()).unwrap_or_default();
                serde_json::json!({
                    "ok": true, "id": id,
                    "result": {
                        "photos": snap.photos,
                        "analyzed": snap.analyzed,
                        "archive": snap.archive,
                        "status": snap.status,
                        "module": snap.module,
                    }
                })
            }
            "openFolder" => match params.get("path").and_then(|p| p.as_str()) {
                Some(path) => {
                    let _ = sender.send(Command::OpenFolder(path.to_string()));
                    serde_json::json!({ "ok": true, "id": id })
                }
                None => serde_json::json!({ "ok": false, "id": id, "error": "manca path" }),
            },
            "analyzeAll" => {
                let _ = sender.send(Command::AnalyzeAll);
                serde_json::json!({ "ok": true, "id": id })
            }
            "select" => match params.get("index").and_then(|i| i.as_u64()) {
                Some(index) => {
                    let _ = sender.send(Command::Select(index as usize));
                    serde_json::json!({ "ok": true, "id": id })
                }
                None => serde_json::json!({ "ok": false, "id": id, "error": "manca index" }),
            },
            "setModule" => match params.get("module").and_then(|m| m.as_str()) {
                Some(name) => {
                    let _ = sender.send(Command::SetModule(name.to_string()));
                    serde_json::json!({ "ok": true, "id": id })
                }
                None => serde_json::json!({ "ok": false, "id": id, "error": "manca module" }),
            },
            "demo" => {
                let _ = sender.send(Command::Demo);
                serde_json::json!({ "ok": true, "id": id })
            }
            "quit" => {
                let _ = sender.send(Command::Quit);
                serde_json::json!({ "ok": true, "id": id })
            }
            other => {
                serde_json::json!({ "ok": false, "id": id, "error": format!("comando sconosciuto: {other}") })
            }
        };
        if method != "status" {
            // un comando è stato accodato: chiedi un ridisegno così l'app lo elabora subito
            ctx.request_repaint();
        }
        if writeln!(writer, "{reply}").is_err() {
            break;
        }
    }
}
