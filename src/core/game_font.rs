use bevy::prelude::*;

use std::path::Path;
use std::sync::OnceLock;

static GLOBAL_FONT: OnceLock<Handle<Font>> = OnceLock::new();

#[derive(Resource, Default)]
pub struct GameFont(pub Handle<Font>);

/// Stocke la police dans un global pour y accéder sans passer par `Res<GameFont>`.
pub fn set_global_font(handle: Handle<Font>) {
    GLOBAL_FONT.set(handle).ok();
}

/// Retourne la police globale, ou `default()` si non définie.
pub fn global_font() -> Handle<Font> {
    GLOBAL_FONT.get().cloned().unwrap_or_default()
}

/// Remplace `TextFont::from_font_size()` dans toutes les UI.
/// Utilise la police globale au lieu de la police Bevy par défaut.
pub fn tf(size: f32) -> TextFont {
    TextFont {
        font: global_font().into(),
        font_size: size.into(),
        ..default()
    }
}

/// Charge la police depuis `assets/fonts/font.ttf`.
/// Si le fichier n'existe pas, utilise la police Bevy par défaut.
pub fn load_game_font(mut font: ResMut<GameFont>, asset_server: Res<AssetServer>) {
    let font_path = Path::new("assets").join("fonts").join("font.ttf");
    if font_path.exists() {
        font.0 = asset_server.load("fonts/font.ttf");
        set_global_font(font.0.clone());
        info!("GameFont: loaded fonts/font.ttf");
    } else {
        error!("GameFont: assets/fonts/font.ttf not found — place a .ttf file there");
        // Fallback: police Bevy par défaut
        set_global_font(font.0.clone());
    }
}
