//! Tooling del progetto: `ci`, `layers`, `assets`, `parity`.
//!
//! È uno strumento di sviluppo, non codice di produzione: qui `expect` è ammesso. Girando con
//! `cargo xtask <comando>` dalla radice del workspace.

use std::path::Path;
use std::process::Command;

/// La tabella degli strati: (percorso, livello). Le dipendenze puntano solo verso il basso.
const CRATES: &[(&str, u8)] = &[
    ("crates/raster", 0),
    ("crates/metadata", 1),
    ("crates/analysis", 1),
    ("crates/archive", 2),
    ("crates/ai", 2),
    ("crates/report", 3),
    ("crates/engine", 4),
    ("crates/ui-egui", 5),
    ("apps/photoinsight", 5),
];

/// Le dipendenze che solo il livello UI (5) può avere.
const UI_ONLY_DEPENDENCIES: &[&str] = &["eframe", "egui", "winit", "egui-winit", "egui_glow"];

/// I documenti che devono esistere.
const DOCS: &[&str] = &[
    "ROADMAP.md",
    "docs/gaps.md",
    "docs/parity-checklist.md",
    "docs/target-app-parity.md",
];

fn main() {
    let command = std::env::args().nth(1).unwrap_or_else(|| "help".to_string());
    let result = match command.as_str() {
        "ci" => ci(),
        "layers" => layers(),
        "assets" => assets(),
        "parity" => parity(),
        "help" | "--help" | "-h" => {
            println!("uso: cargo xtask <ci|layers|assets|parity>");
            Ok(())
        }
        other => Err(format!("comando sconosciuto: {other} (prova `help`)")),
    };
    if let Err(message) = result {
        eprintln!("xtask: {message}");
        std::process::exit(1);
    }
}

/// Esegue un comando e fallisce se esce con errore.
fn run(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|error| format!("non riesco a eseguire {program}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`{program} {}` è fallito", args.join(" ")))
    }
}

/// Tutti i gate, nell'ordine: formattazione, lint, test, strati, asset.
fn ci() -> Result<(), String> {
    run("cargo", &["fmt", "--all", "--check"])?;
    run(
        "cargo",
        &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
    )?;
    run("cargo", &["test", "--workspace"])?;
    layers()?;
    assets()?;
    parity()?;
    println!("\nOK: tutti i gate sono verdi.");
    Ok(())
}

/// Verifica che nessuna crate sotto il livello UI dipenda da una libreria di interfaccia.
fn layers() -> Result<(), String> {
    for (path, level) in CRATES {
        if *level >= 5 {
            continue;
        }
        let manifest = Path::new(path).join("Cargo.toml");
        let text =
            std::fs::read_to_string(&manifest).map_err(|error| format!("{}: {error}", manifest.display()))?;
        for line in text.lines() {
            let trimmed = line.trim();
            for dependency in UI_ONLY_DEPENDENCIES {
                let is_dependency = trimmed
                    .strip_prefix(dependency)
                    .map(|rest| rest.trim_start().starts_with('='))
                    .unwrap_or(false);
                if is_dependency {
                    return Err(format!(
                        "{path} (livello {level}) dipende da `{dependency}`: la UI sta troppo in basso"
                    ));
                }
            }
        }
    }
    println!("OK: nessuna violazione degli strati ({} crate).", CRATES.len());
    Ok(())
}

/// Verifica che ogni file sotto `assets/` sia registrato in `assets/ATTRIBUTION.md`.
fn assets() -> Result<(), String> {
    let attribution = Path::new("assets/ATTRIBUTION.md");
    let text = std::fs::read_to_string(attribution)
        .map_err(|error| format!("{}: {error}", attribution.display()))?;
    let mut missing = Vec::new();
    let mut stack = vec![Path::new("assets").to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).map_err(|error| format!("{}: {error}", dir.display()))? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().and_then(|n| n.to_str()) != Some("ATTRIBUTION.md") {
                let relative = path.to_string_lossy().replace('\\', "/");
                if !text.contains(&relative) {
                    missing.push(relative);
                }
            }
        }
    }
    if missing.is_empty() {
        println!("OK: ogni asset in assets/ è attribuito.");
        Ok(())
    } else {
        Err(format!(
            "asset senza voce in ATTRIBUTION.md: {}",
            missing.join(", ")
        ))
    }
}

/// Verifica che i documenti di avanzamento esistano.
fn parity() -> Result<(), String> {
    for doc in DOCS {
        if !Path::new(doc).exists() {
            return Err(format!("documento mancante: {doc}"));
        }
    }
    println!("OK: i documenti di avanzamento ci sono ({}).", DOCS.len());
    Ok(())
}
