//! Buffer di pixel RGBA8, condiviso da tutte le crate che toccano immagini.
//!
//! Convenzione del progetto: commenti e docstring in italiano, identificatori in inglese.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

use std::path::Path;

/// Un'immagine RGBA8, righe dall'alto verso il basso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgba8 {
    width: usize,
    height: usize,
    data: Vec<[u8; 4]>,
}

impl Rgba8 {
    /// Crea un'immagine dai pixel. Restituisce `None` se le dimensioni non tornano: l'input è ostile.
    pub fn new(width: usize, height: usize, data: Vec<[u8; 4]>) -> Option<Self> {
        if width == 0 || height == 0 || width.checked_mul(height) != Some(data.len()) {
            return None;
        }
        Some(Self { width, height, data })
    }

    /// Un'immagine tinta di un solo colore.
    pub fn filled(width: usize, height: usize, rgba: [u8; 4]) -> Option<Self> {
        let count = width.checked_mul(height)?;
        if count == 0 {
            return None;
        }
        Some(Self {
            width,
            height,
            data: vec![rgba; count],
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn pixels(&self) -> &[[u8; 4]] {
        &self.data
    }

    /// Il pixel a `(x, y)`, se dentro i bordi. Mai indicizzazione diretta su valori dall'esterno.
    pub fn pixel(&self, x: usize, y: usize) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = y.checked_mul(self.width)?.checked_add(x)?;
        self.data.get(index).copied()
    }
}

/// Cosa può andare storto decodificando un'immagine.
#[derive(Debug)]
pub enum RasterError {
    /// Il file non si è potuto aprire o leggere.
    Io(String),
    /// Il formato c'è ma i pixel non si sono potuti decodificare.
    Decode(String),
    /// L'immagine decodificata non ha pixel.
    Empty,
}

impl std::fmt::Display for RasterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(what) => write!(f, "errore di lettura: {what}"),
            Self::Decode(what) => write!(f, "immagine non decodificabile: {what}"),
            Self::Empty => write!(f, "l'immagine non ha pixel"),
        }
    }
}

impl std::error::Error for RasterError {}

/// Le dimensioni dichiarate da un file immagine, senza decodificarlo tutto.
pub fn dimensions(path: &Path) -> Result<(u32, u32), RasterError> {
    image::image_dimensions(path).map_err(|error| RasterError::Decode(error.to_string()))
}

/// Decodifica un file immagine e ne restituisce una miniatura RGBA8.
///
/// Il lato più lungo non supera `max_edge`: l'input è ostile e non si alloca memoria per un'immagine
/// che dichiara dimensioni enormi.
pub fn decode_thumbnail(path: &Path, max_edge: u32) -> Result<Rgba8, RasterError> {
    let reader = image::ImageReader::open(path).map_err(|error| RasterError::Io(error.to_string()))?;
    let reader = reader
        .with_guessed_format()
        .map_err(|error| RasterError::Io(error.to_string()))?;
    let decoded = reader
        .decode()
        .map_err(|error| RasterError::Decode(error.to_string()))?;
    let edge = max_edge.clamp(1, 8192);
    let thumb = decoded.thumbnail(edge, edge);
    let rgba = thumb.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());
    let data = rgba
        .pixels()
        .map(|p| [p[0], p[1], p[2], p[3]])
        .collect::<Vec<_>>();
    Rgba8::new(width as usize, height as usize, data).ok_or(RasterError::Empty)
}

/// I pixel di un'immagine come byte RGBA in sequenza, pronti per una texture.
pub fn rgba_bytes(image: &Rgba8) -> Vec<u8> {
    let mut out = Vec::with_capacity(image.data.len() * 4);
    for pixel in &image.data {
        out.extend_from_slice(pixel);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_image_keeps_its_pixels() {
        let img = Rgba8::new(2, 1, vec![[1, 2, 3, 4], [5, 6, 7, 8]]).expect("2x1 con due pixel");
        assert_eq!(img.width(), 2);
        assert_eq!(img.pixel(0, 0), Some([1, 2, 3, 4]));
        assert_eq!(img.pixel(1, 0), Some([5, 6, 7, 8]));
        assert_eq!(img.pixel(2, 0), None);
        assert_eq!(img.pixel(0, 1), None);
    }

    #[test]
    fn a_wrong_pixel_count_is_refused_not_allocated() {
        assert!(Rgba8::new(2, 2, vec![[0, 0, 0, 0]]).is_none());
        assert!(Rgba8::new(0, 1, vec![]).is_none());
        assert!(Rgba8::new(usize::MAX, 2, vec![]).is_none());
    }

    #[test]
    fn a_filled_image_has_the_colour_everywhere() {
        let img = Rgba8::filled(3, 2, [9, 8, 7, 6]).expect("3x2");
        assert_eq!(img.pixels().len(), 6);
        assert_eq!(img.pixel(2, 1), Some([9, 8, 7, 6]));
    }
}
