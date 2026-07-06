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

/// Convert tile grid coordinates to world position (center of tile).
pub fn tile_to_world(tx: i32, ty: i32, tile_size: f32) -> Vec2 {
    Vec2::new(
        tx as f32 * tile_size + tile_size / 2.0,
        ty as f32 * tile_size + tile_size / 2.0,
    )
}

/// Convert tile grid coordinates to world position (bottom-left corner of tile).
pub fn tile_to_world_corner(tx: i32, ty: i32, tile_size: f32) -> Vec2 {
    Vec2::new(
        tx as f32 * tile_size - tile_size / 2.0,
        ty as f32 * tile_size - tile_size / 2.0,
    )
}

/// Convert world position to tile grid coordinates.
pub fn world_to_tile(pos: Vec2, tile_size: f32) -> (i32, i32) {
    let tx = ((pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let ty = ((pos.y + tile_size / 2.0) / tile_size).floor() as i32;
    (tx, ty)
}

/// Move `translation` toward `target` at `speed` on a 2D plane (x/y), returns true when arrived.
pub fn move_toward(translation: &mut Vec3, target: Vec3, speed: f32, dt: f32) -> bool {
    let dx = target.x - translation.x;
    let dy = target.y - translation.y;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 0.001 {
        return true;
    }
    let step = (speed * dt).min(dist);
    translation.x += dx / dist * step;
    translation.y += dy / dist * step;
    false
}

/// Load a TOML file embedded at compile time into the target type.
/// Panics with a clear message on parse failure.
#[macro_export]
macro_rules! load_toml {
    ($path:literal, $ty:ty) => {{
        let toml_str = include_str!($path);
        toml::from_str::<$ty>(toml_str).unwrap_or_else(|e| {
            panic!("failed to parse {}: {}", $path, e)
        })
    }};
}
