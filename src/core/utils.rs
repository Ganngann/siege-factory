use bevy::prelude::*;
use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("siege-factory");
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg).join("siege-factory");
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(".config").join("siege-factory");
        }
    }
    PathBuf::from(".").join("config")
}

pub fn parse_hex_color(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    if s.len() < 6 {
        return Color::srgb(0.5, 0.5, 0.5);
    }
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(128) as f32 / 255.0;
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(128) as f32 / 255.0;
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(128) as f32 / 255.0;
    Color::srgb(r, g, b)
}

pub fn silent_despawn(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).try_despawn();
}
