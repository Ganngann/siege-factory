use bevy::prelude::*;
use serde::Deserialize;

/// Thème 100% data-driven. Chargeable depuis `theme.toml`, mergeable entre mods.
/// Si un mod ne définit pas de thème, les valeurs par défaut s'appliquent.
#[derive(Debug, Clone, Resource)]
pub struct Theme {
    pub window_bg: Color,
    pub section_bg: Color,
    pub panel_overlay: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_green: Color,
    pub text_yellow: Color,
    pub accent: Color,
    pub btn_close: Color,
    pub btn_active: Color,
    pub btn_inactive: Color,
    pub btn_hover: Color,
    pub hp_bar: Color,
    pub bar_bg: Color,
    pub progress_fill: Color,
    pub border: Color,
    pub font_size_title: f32,
    pub font_size_medium: f32,
    pub font_size_body: f32,
    pub font_size_small: f32,
    pub font_size_label: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            window_bg: Color::srgba(0.08, 0.08, 0.16, 0.97),
            section_bg: Color::srgb(0.10, 0.10, 0.18),
            panel_overlay: Color::srgba(0.0, 0.0, 0.0, 0.45),
            text_primary: Color::srgb(0.90, 0.90, 1.00),
            text_secondary: Color::srgb(0.60, 0.60, 0.75),
            text_green: Color::srgb(0.40, 0.85, 0.40),
            text_yellow: Color::srgb(0.85, 0.85, 0.35),
            accent: Color::srgb(0.30, 0.55, 1.00),
            btn_close: Color::srgb(0.50, 0.12, 0.12),
            btn_active: Color::srgb(0.15, 0.45, 0.15),
            btn_inactive: Color::srgb(0.30, 0.15, 0.15),
            btn_hover: Color::srgb(0.25, 0.30, 0.45),
            hp_bar: Color::srgb(0.20, 0.65, 0.20),
            bar_bg: Color::srgb(0.15, 0.15, 0.22),
            progress_fill: Color::srgb(0.30, 0.55, 1.00),
            border: Color::srgb(0.20, 0.20, 0.30),
            font_size_title: 16.0,
            font_size_medium: 14.0,
            font_size_body: 12.0,
            font_size_small: 10.0,
            font_size_label: 11.0,
        }
    }
}

#[derive(Deserialize)]
struct ThemeToml {
    #[serde(default)]
    window_bg: Option<String>,
    #[serde(default)]
    section_bg: Option<String>,
    #[serde(default)]
    text_primary: Option<String>,
    #[serde(default)]
    text_secondary: Option<String>,
    #[serde(default)]
    accent: Option<String>,
    #[serde(default)]
    btn_close: Option<String>,
    #[serde(default)]
    font_size_body: Option<f32>,
    #[serde(default)]
    font_size_medium: Option<f32>,
}

impl Theme {
    pub fn load(mods: &crate::core::modding::ModRegistry) -> Self {
        let mut theme = Self::default();
        for (_mod_id, content) in mods.load_all_data("ui/theme.toml") {
            if let Ok(parsed) = toml::from_str::<ThemeToml>(&content) {
                theme.apply_overrides(parsed);
            }
        }
        theme
    }

    fn apply_overrides(&mut self, t: ThemeToml) {
        if let Some(c) = t.window_bg { self.window_bg = parse_hex(&c); }
        if let Some(c) = t.section_bg { self.section_bg = parse_hex(&c); }
        if let Some(c) = t.text_primary { self.text_primary = parse_hex(&c); }
        if let Some(c) = t.text_secondary { self.text_secondary = parse_hex(&c); }
        if let Some(c) = t.accent { self.accent = parse_hex(&c); }
        if let Some(c) = t.btn_close { self.btn_close = parse_hex(&c); }
        if let Some(s) = t.font_size_body { self.font_size_body = s; }
        if let Some(s) = t.font_size_medium { self.font_size_medium = s; }
    }
}

fn parse_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
    Color::srgb(r, g, b)
}
