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
