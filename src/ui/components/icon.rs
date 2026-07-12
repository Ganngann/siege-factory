use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::context::UiDataContext;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::registry::ComponentRegistry;
use crate::ui::theme::Theme;

fn parse_hex(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
    } else {
        None
    }
}

fn icon_char(name: &str) -> &'static str {
    match name {
        "warning" => "⚠",
        "alert" => "⚡",
        "check" => "✔",
        "cross" => "✘",
        "info" => "ℹ",
        "heart" => "♥",
        "power" => "⚡",
        _ => "?",
    }
}

fn icon_color(name: &str) -> Color {
    match name {
        "red" => Color::srgb(1.0, 0.2, 0.2),
        "green" => Color::srgb(0.2, 1.0, 0.2),
        "yellow" => Color::srgb(1.0, 0.8, 0.2),
        _ => Color::srgb(0.60, 0.60, 0.75),
    }
}

pub struct IconComponent;
impl UiComponent for IconComponent {
    fn id(&self) -> &str { "icon" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, data: &UiDataContext, _theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        let name = config.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let size = config.get("size").and_then(|v| v.as_float()).unwrap_or(16.0) as f32;
        let color_str = config.get("color").and_then(|v| v.as_str()).unwrap_or("currentColor");
        let color = if color_str == "currentColor" {
            Color::srgb(0.60, 0.60, 0.75)
        } else if color_str.starts_with('#') {
            parse_hex(color_str).unwrap_or(Color::srgb(0.60, 0.60, 0.75))
        } else if color_str.starts_with("capsule.") {
            let hex = data.resolve(color_str);
            parse_hex(&hex).unwrap_or(Color::srgb(0.60, 0.60, 0.75))
        } else {
            icon_color(color_str)
        };

        spawn_child(commands, parent, (
            Text::new(icon_char(name).to_string()),
            tf(size),
            TextColor(color),
        ))
    }
}
