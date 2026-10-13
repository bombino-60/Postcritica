//! La palette e le misure del tema, **copiate dal riferimento LightCraft**.
//!
//! I valori sono presi dal tema dell'app di riferimento (osservazione dell'interfaccia, tutto codice
//! nostro). L'applicazione ai `Visuals` di egui riproduce la stessa ricetta.

use eframe::egui;
use eframe::egui::{Color32, CornerRadius, Stroke, Visuals};

/// I colori e le misure del tema.
///
/// È la tabella completa del riferimento: alcuni valori servono a tappe successive.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Tokens {
    /// Barra in alto, pannelli laterali, barra in basso, striscia dei moduli.
    pub chrome: Color32,
    /// La tela della foto e il filmstrip.
    pub canvas: Color32,
    /// Lo sfondo dietro le celle della griglia.
    pub grid_bg: Color32,
    pub cell: Color32,
    pub cell_selected: Color32,
    pub divider: Color32,
    pub inset: Color32,
    pub field: Color32,
    pub field_border: Color32,
    pub button: Color32,
    pub button_border: Color32,
    pub hover: Color32,
    pub pressed: Color32,
    pub tool_active: Color32,
    pub text: Color32,
    pub text_label: Color32,
    pub text_dim: Color32,
    pub text_disabled: Color32,
    /// I glifi delle icone nella chrome.
    pub icon: Color32,
    pub track: Color32,
    pub thumb: Color32,
    pub thumb_hover: Color32,
    pub accent: Color32,
    pub star: Color32,
    pub pick: Color32,
    pub reject: Color32,
    /// Avvisi (per esempio un RAW mostrato dall'anteprima incorporata): un ambra sommesso.
    pub caution: Color32,
    /// Un verde per i giudizi positivi (il verde delle etichette del riferimento).
    pub positive: Color32,
    pub mask_overlay: Color32,
    // misure (punti)
    pub top_bar_h: f32,
    pub bottom_bar_h: f32,
    pub panel_w: f32,
    pub strip_w: f32,
}

impl Default for Tokens {
    fn default() -> Self {
        Tokens {
            chrome: Color32::from_rgb(0x2d, 0x2d, 0x2d),
            canvas: Color32::from_rgb(0x1c, 0x1c, 0x1c),
            grid_bg: Color32::from_rgb(0x0f, 0x0f, 0x0f),
            cell: Color32::from_rgb(0x1c, 0x1c, 0x1c),
            cell_selected: Color32::from_rgb(0x2d, 0x2d, 0x2d),
            divider: Color32::from_rgb(0x1c, 0x1c, 0x1c),
            inset: Color32::from_rgb(0x23, 0x23, 0x23),
            field: Color32::from_rgb(0x22, 0x22, 0x22),
            field_border: Color32::from_rgb(0x3c, 0x3c, 0x3c),
            button: Color32::from_rgb(0x24, 0x24, 0x24),
            button_border: Color32::from_rgb(0x4c, 0x4c, 0x4c),
            hover: Color32::from_rgb(0x3a, 0x3a, 0x3a),
            pressed: Color32::from_rgb(0x3f, 0x3f, 0x3f),
            tool_active: Color32::from_rgb(0x3f, 0x3f, 0x3f),
            text: Color32::from_rgb(0xe2, 0xe2, 0xe2),
            text_label: Color32::from_rgb(0xbc, 0xbc, 0xbc),
            text_dim: Color32::from_rgb(0x8e, 0x8e, 0x8e),
            text_disabled: Color32::from_rgb(0x5c, 0x5c, 0x5c),
            icon: Color32::from_rgb(0x9a, 0x9a, 0x9a),
            track: Color32::from_rgb(0x5a, 0x5a, 0x5a),
            thumb: Color32::from_rgb(0xa0, 0xa0, 0xa0),
            thumb_hover: Color32::from_rgb(0xe0, 0xe0, 0xe0),
            accent: Color32::from_rgb(0x01, 0x65, 0xdd),
            star: Color32::from_rgb(0xd8, 0xd8, 0xd8),
            pick: Color32::from_rgb(0xf0, 0xf0, 0xf0),
            reject: Color32::from_rgb(0xe0, 0x4a, 0x4a),
            caution: Color32::from_rgb(0xe3, 0xa8, 0x3c),
            positive: Color32::from_rgb(0x58, 0xb0, 0x5c),
            mask_overlay: Color32::from_rgba_unmultiplied(0xe0, 0x20, 0x30, 110),
            top_bar_h: 42.0,
            bottom_bar_h: 48.0,
            panel_w: 270.0,
            strip_w: 48.0,
        }
    }
}

impl Tokens {
    /// I token correnti (o quelli predefiniti).
    pub fn get(ctx: &egui::Context) -> Tokens {
        ctx.data(|data| data.get_temp::<Tokens>(egui::Id::NULL))
            .unwrap_or_default()
    }

    /// Il colore di un punteggio, dal verde al rosso.
    pub fn score_colour(&self, score: Option<i32>) -> Color32 {
        match score {
            Some(value) if value >= 75 => self.positive,
            Some(value) if value >= 50 => self.caution,
            Some(_) => self.reject,
            None => self.text_dim,
        }
    }
}

/// Applica il tema: colori e misure, nello stesso modo del riferimento.
pub fn apply(ctx: &egui::Context) {
    let tokens = Tokens::default();
    ctx.data_mut(|data| data.insert_temp(egui::Id::NULL, tokens));

    let mut visuals = Visuals::dark();
    visuals.panel_fill = tokens.chrome;
    visuals.window_fill = tokens.chrome;
    visuals.extreme_bg_color = tokens.field;
    visuals.faint_bg_color = tokens.inset;
    visuals.window_stroke = Stroke::new(1.0, tokens.button_border);
    visuals.window_corner_radius = CornerRadius::same(6);
    visuals.menu_corner_radius = CornerRadius::same(6);
    visuals.selection.bg_fill = tokens.accent.gamma_multiply(0.6);
    visuals.selection.stroke = Stroke::new(1.0, tokens.accent);
    visuals.override_text_color = Some(tokens.text_label);
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(140),
    };
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 8],
        blur: 30,
        spread: 0,
        color: Color32::from_black_alpha(160),
    };

    let widgets = &mut visuals.widgets;
    let paint = |state: &mut egui::style::WidgetVisuals, fill: Color32| {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = Stroke::new(1.0, tokens.text_label);
        state.fg_stroke = Stroke::new(1.0, tokens.text_label);
        state.corner_radius = CornerRadius::same(4);
    };
    paint(&mut widgets.noninteractive, tokens.chrome);
    paint(&mut widgets.inactive, tokens.button);
    paint(&mut widgets.hovered, tokens.hover);
    paint(&mut widgets.active, tokens.pressed);
    paint(&mut widgets.open, tokens.hover);
    widgets.noninteractive.bg_stroke = Stroke::new(1.0, tokens.divider);
    widgets.inactive.bg_stroke = Stroke::new(1.0, tokens.button_border);

    ctx.set_visuals(visuals);
    ctx.global_style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        style.spacing.interact_size.y = 26.0;
        style.spacing.slider_width = 160.0;
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(15.0));
        style
            .text_styles
            .insert(egui::TextStyle::Small, egui::FontId::proportional(12.5));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(20.0));
        style
            .text_styles
            .insert(egui::TextStyle::Monospace, egui::FontId::monospace(14.0));
        style.animation_time = 0.08;
    });
}
