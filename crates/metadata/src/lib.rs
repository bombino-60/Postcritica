//! Lettura e scrittura dei metadati di una foto: EXIF, XMP, IPTC.
//!
//! Il port parte dai formati comuni (JPEG / TIFF / HEIC / PNG / WebP); i RAW arrivano quando c'è una
//! crate Rust adatta. Fino ad allora un RAW risponde "formato non supportato", senza mai andare in panic.

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
    /// La data di scatto, come la scrive la fotocamera (`AAAA-MM-GG hh:mm:ss`).
    pub shot_at: Option<String>,
    /// Latitudine e longitudine, in gradi decimali.
    pub gps: Option<(f64, f64)>,
    pub rating: Option<i32>,
    pub keywords: Vec<String>,
}

/// Cosa può andare storto leggendo o scrivendo i metadati.
#[derive(Debug)]
pub enum MetadataError {
    /// Formato non ancora supportato (il port parte da JPEG / TIFF / HEIC / PNG / WebP).
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

/// Le estensioni che il lettore prova ad aprire.
const SUPPORTED: &[&str] = &["jpg", "jpeg", "tif", "tiff", "heic", "heif", "png", "webp"];

/// Legge i metadati di una foto.
///
/// Un'immagine senza EXIF (o con EXIF illeggibile) non è un errore: restituisce metadati vuoti. Solo un
/// formato non supportato o un problema di I/O sono errori.
pub fn read(path: &Path) -> Result<PhotoMetadata, MetadataError> {
    std::fs::metadata(path)?; // un percorso assente è un errore di I/O, non un panic
    let ext = extension_of(path);
    if !SUPPORTED.contains(&ext.as_str()) {
        return Err(MetadataError::Unsupported(format!("estensione .{ext}")));
    }
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let exif = match exif::Reader::new().read_from_container(&mut reader) {
        Ok(exif) => exif,
        // senza EXIF, o con EXIF che non si riesce a leggere: nessun metadato, ma nessun errore
        Err(_) => return Ok(PhotoMetadata::default()),
    };

    let make = field_string(&exif, exif::Tag::Make);
    let model = field_string(&exif, exif::Tag::Model);
    let camera = match (make, model) {
        (Some(make), Some(model)) => Some(format!("{make} {model}")),
        (Some(make), None) => Some(make),
        (None, Some(model)) => Some(model),
        (None, None) => None,
    };

    Ok(PhotoMetadata {
        camera,
        lens: field_string(&exif, exif::Tag::LensModel),
        iso: field_u32(&exif, exif::Tag::PhotographicSensitivity),
        aperture: field_rational(&exif, exif::Tag::FNumber),
        shutter: field_rational_pair(&exif, exif::Tag::ExposureTime)
            .map(|(num, den)| format_shutter(num, den)),
        focal_mm: field_rational(&exif, exif::Tag::FocalLength),
        shot_at: field_string(&exif, exif::Tag::DateTimeOriginal),
        gps: None,
        // il rating vive nell'XMP, non nell'EXIF: arriverà con la lettura XMP
        rating: None,
        keywords: Vec::new(),
    })
}

fn extension_of(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// Il testo di un campo EXIF, con le virgolette che il display aggiunge rimosse.
fn field_string(exif: &exif::Exif, tag: exif::Tag) -> Option<String> {
    let field = exif.get_field(tag, exif::In::PRIMARY)?;
    let text = field.display_value().to_string();
    let trimmed = text.trim().trim_matches('"').trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn field_u32(exif: &exif::Exif, tag: exif::Tag) -> Option<u32> {
    let field = exif.get_field(tag, exif::In::PRIMARY)?;
    match &field.value {
        exif::Value::Short(values) => values.first().map(|v| u32::from(*v)),
        exif::Value::Long(values) => values.first().copied(),
        other => other.display_as(tag).to_string().trim().parse::<u32>().ok(),
    }
}

/// Il valore di un campo razionale come numero decimale.
fn field_rational(exif: &exif::Exif, tag: exif::Tag) -> Option<f64> {
    let (num, den) = field_rational_pair(exif, tag)?;
    if den == 0 {
        return None;
    }
    Some(f64::from(num) / f64::from(den))
}

/// Il valore di un campo razionale come coppia numeratore/denominatore.
fn field_rational_pair(exif: &exif::Exif, tag: exif::Tag) -> Option<(u32, u32)> {
    let field = exif.get_field(tag, exif::In::PRIMARY)?;
    match &field.value {
        exif::Value::Rational(values) => values.first().map(|r| (r.num, r.denom)),
        _ => None,
    }
}

/// Formatta il tempo di scatto nella forma consueta (`1/250`).
fn format_shutter(num: u32, den: u32) -> String {
    if den == 0 {
        return String::from("?");
    }
    if num == 1 {
        format!("1/{den}")
    } else if num == den {
        String::from("1 s")
    } else {
        let seconds = f64::from(num) / f64::from(den);
        format!("{seconds:.3} s")
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
    fn an_unknown_extension_says_which_one() {
        let path = std::env::temp_dir().join("postcritica-t2-probe.xyz");
        std::fs::write(&path, b"qualunque cosa").expect("scrivo il file di prova");
        let result = read(&path);
        let _ = std::fs::remove_file(&path);
        assert!(
            matches!(&result, Err(MetadataError::Unsupported(what)) if what.contains("xyz")),
            "atteso Unsupported con 'xyz', trovato {result:?}"
        );
    }

    #[test]
    fn an_image_without_exif_is_ok_and_empty() {
        let path = std::env::temp_dir().join("postcritica-t2-noexif.png");
        image::RgbaImage::from_pixel(2, 2, image::Rgba([10, 20, 30, 255]))
            .save(&path)
            .expect("scrivo il png");
        let result = read(&path);
        let _ = std::fs::remove_file(&path);
        assert_eq!(result.expect("nessun errore"), PhotoMetadata::default());
    }
}
