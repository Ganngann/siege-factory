use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::context::UiDataContext;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::theme::Theme;
use crate::ui::registry::ComponentRegistry;

pub struct HudTextComponent;
impl UiComponent for HudTextComponent {
    fn id(&self) -> &str { "hud_text" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &UiDataContext, _theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        let data_key = config.get("data_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let font_size = config.get("font_size").and_then(|v| v.as_float()).unwrap_or(14.0) as f32;
        let color_hex = config.get("color").and_then(|v| v.as_str()).unwrap_or("#ffffff");
        let color = parse_hex(color_hex);
        let top = config.get("position").and_then(|v| v.get("top")).and_then(|v| v.as_float()).unwrap_or(0.0);
        let right = config.get("position").and_then(|v| v.get("right")).and_then(|v| v.as_float());
        let left = config.get("position").and_then(|v| v.get("left")).and_then(|v| v.as_float());
        let bottom = config.get("position").and_then(|v| v.get("bottom")).and_then(|v| v.as_float());

        let mut node = Node {
            position_type: PositionType::Absolute,
            top: Val::Px(top),
            padding: UiRect::all(Val::Px(4.0)),
            ..default()
        };
        if let Some(r) = right { node.right = Val::Px(r); }
        if let Some(l) = left { node.left = Val::Px(l); }
        if let Some(b) = bottom { node.bottom = Val::Px(b); }

        spawn_child(commands, parent, (
            Text::new(String::new()),
            tf(font_size),
            TextColor(color),
            node,
            HudText { data_key },
        ))
    }
}

#[derive(Component, Clone)]
pub struct HudText {
    pub data_key: String,
}

fn parse_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
        Color::srgb(r, g, b)
    } else {
        Color::WHITE
    }
}
