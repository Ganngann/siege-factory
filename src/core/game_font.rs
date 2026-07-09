use bevy::prelude::*;
use std::path::Path;

#[derive(Resource, Default)]
pub struct GameFont(pub Handle<Font>);

/// Charge la police depuis `assets/fonts/font.ttf`.
/// Si le fichier n'existe pas, utilise la police Bevy par défaut.
pub fn load_game_font(mut font: ResMut<GameFont>, asset_server: Res<AssetServer>) {
    let font_path = Path::new("assets").join("fonts").join("font.ttf");
    if font_path.exists() {
        font.0 = asset_server.load("fonts/font.ttf");
        info!("GameFont: loaded fonts/font.ttf");
    } else {
        error!("GameFont: assets/fonts/font.ttf not found — run cargo build to download it");
        // Fallback: police Bevy par défaut (peut manquer d'accents)
    }
}
