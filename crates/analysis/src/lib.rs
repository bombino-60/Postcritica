//! Ispezione locale di un'immagine: profilo visivo e indicatori tecnici.
//!
//! Come `insight_engine.py` del programma Python, questo modulo **non fa rete**: legge solo i pixel
//! già decodificati e ne ricava numeri che servono sia alla UI sia al prompt del valutatore.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

use postcritica_raster::Rgba8;

/// Statistiche di base di un'immagine, in scala 0..1.
#[derive(Debug, Clone, PartialEq)]
pub struct VisualProfile {
    /// Luminanza media.
    pub brightness: f64,
    /// Deviazione standard della luminanza: una misura grezza di contrasto.
    pub contrast: f64,
}

impl VisualProfile {
    /// Calcola il profilo da un'immagine già decodificata.
    pub fn of(image: &Rgba8) -> Self {
        let pixels = image.pixels();
        let count = pixels.len();
        if count == 0 {
            return Self {
                brightness: 0.0,
                contrast: 0.0,
            };
        }
        let mean = pixels.iter().map(|p| luma(*p)).sum::<f64>() / count as f64;
        let variance = pixels.iter().map(|p| (luma(*p) - mean).powi(2)).sum::<f64>() / count as f64;
        Self {
            brightness: mean,
            contrast: variance.sqrt(),
        }
    }
}

/// Luminanza Rec.601 di un pixel RGBA, normalizzata a 0..1.
fn luma(pixel: [u8; 4]) -> f64 {
    (0.299 * f64::from(pixel[0]) + 0.587 * f64::from(pixel[1]) + 0.114 * f64::from(pixel[2])) / 255.0
}

/// Rischio di micromosso con la regola del reciproco: il tempo di scatto è più lungo di 1/focale.
///
/// Restituisce `None` se i due valori non sono utilizzabili (non finiti, zero o negativi).
pub fn shake_risk(shutter_seconds: f64, focal_mm: f64) -> Option<bool> {
    if !shutter_seconds.is_finite() || !focal_mm.is_finite() || shutter_seconds <= 0.0 || focal_mm <= 0.0 {
        return None;
    }
    Some(shutter_seconds > 1.0 / focal_mm)
}

/// Rischio di rumore visibile a partire dagli ISO. `None` sotto la soglia di guardia.
pub fn noise_risk(iso: u32) -> Option<bool> {
    if iso == 0 {
        return None;
    }
    Some(iso >= 1600)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_and_white_pin_the_profile_at_the_ends() {
        let black = Rgba8::filled(4, 4, [0, 0, 0, 255]).expect("4x4");
        let white = Rgba8::filled(4, 4, [255, 255, 255, 255]).expect("4x4");
        let b = VisualProfile::of(&black);
        let w = VisualProfile::of(&white);
        assert!((b.brightness - 0.0).abs() < 1e-9);
        assert!((w.brightness - 1.0).abs() < 1e-9);
        assert!(b.contrast.abs() < 1e-9);
        assert!(w.contrast.abs() < 1e-9);
    }

    #[test]
    fn a_half_black_half_white_image_has_contrast() {
        let img = Rgba8::new(2, 1, vec![[0, 0, 0, 255], [255, 255, 255, 255]]).expect("2x1");
        let profile = VisualProfile::of(&img);
        assert!((profile.brightness - 0.5).abs() < 1e-9);
        assert!(profile.contrast > 0.4);
    }

    #[test]
    fn shake_risk_uses_the_reciprocal_rule() {
        // 1/30 s con un 50 mm: 1/30 > 1/50, quindi rischio.
        assert_eq!(shake_risk(1.0 / 30.0, 50.0), Some(true));
        // 1/250 s con un 50 mm: nessun rischio.
        assert_eq!(shake_risk(1.0 / 250.0, 50.0), Some(false));
        // valori ostili: nessun panico, nessun verdetto.
        assert_eq!(shake_risk(0.0, 50.0), None);
        assert_eq!(shake_risk(f64::NAN, 50.0), None);
    }

    #[test]
    fn noise_risk_starts_at_high_iso() {
        assert_eq!(noise_risk(100), Some(false));
        assert_eq!(noise_risk(3200), Some(true));
        assert_eq!(noise_risk(0), None);
    }
}
