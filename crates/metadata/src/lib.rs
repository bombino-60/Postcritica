//! Lettura e scrittura dei metadati di una foto: EXIF, XMP, IPTC.
//!
//! Il port inizia dai formati comuni (JPEG / PNG / TIFF); i RAW arrivano quando c'è una crate Rust
//! adatta. Fino ad allora un RAW risponde "formato non supportato", senza mai andare in panic.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

use std::io;
use std::path::Path;

/// I campi di una foto che il programma mostra e usa.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PhotoMetadata {
    pub camera: Option<String>,
    pub lens: Option<String>,
    pub iso: Option<u32>,
    pub aperture: Option<f64>,
    pub shutter: Option<String>,
    pub focal_mm: Option<f64>,
    /// Latitudine e longitudine, in gradi decimali.
    pub gps: Option<(f64, f64)>,
    pub rating: Option<i32>,
    pub keywords: Vec<String>,
}

/// Cosa può andare storto leggendo o scrivendo i metadati.
#[derive(Debug)]
pub enum MetadataError {
    /// Formato non ancora supportato (il port parte da JPEG / PNG / TIFF).
    Unsupported(String),
    /// Il file non si è potuto leggere.
    Io(io::Error),
    /// I metadati ci sono ma non sono leggibili.
    Malformed(String),
}

impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported(what) => write!(f, "formato non supportato: {what}"),
            Self::Io(error) => write!(f, "errore di lettura: {error}"),
            Self::Malformed(what) => write!(f, "metadati non leggibili: {what}"),
        }
    }
}

impl std::error::Error for MetadataError {}

impl From<io::Error> for MetadataError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Legge i metadati di una foto.
///
/// Oggi distingue i formati supportati da quelli no; la lettura EXIF/XMP vera arriva nella tappa T2.
pub fn read(path: &Path) -> Result<PhotoMetadata, MetadataError> {
    // Il file deve almeno esistere: un percorso assente è un errore di I/O, non un panic.
    let _meta = std::fs::metadata(path)?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "tif" | "tiff" => Err(MetadataError::Unsupported(
            "lettura EXIF non ancora implementata (tappa T2)".to_string(),
        )),
        other => Err(MetadataError::Unsupported(format!("estensione .{other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_is_an_io_error_not_a_panic() {
        let missing = Path::new("questa/immagine/non/esiste.jpg");
        assert!(matches!(read(missing), Err(MetadataError::Io(_))));
    }

    #[test]
    fn a_supported_extension_is_reported_as_not_yet_implemented() {
        let dir = std::env::temp_dir();
        let path = dir.join("postcritica-t2-probe.jpg");
        std::fs::write(&path, b"non e' un vero jpeg").expect("scrivo il file di prova");
        let result = read(&path);
        let _ = std::fs::remove_file(&path);
        assert!(matches!(result, Err(MetadataError::Unsupported(_))));
    }

    #[test]
    fn an_unknown_extension_says_which_one() {
        let dir = std::env::temp_dir();
        let path = dir.join("postcritica-t2-probe.xyz");
        std::fs::write(&path, b"qualunque cosa").expect("scrivo il file di prova");
        let result = read(&path);
        let _ = std::fs::remove_file(&path);
        assert!(
            matches!(&result, Err(MetadataError::Unsupported(what)) if what.contains("xyz")),
            "atteso Unsupported con 'xyz', trovato {result:?}"
        );
    }
}
