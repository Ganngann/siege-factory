use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::core::modding::ModRegistry;

// ── Config resource (loaded from panel_tooltip.toml) ──

#[derive(Resource)]
pub struct TooltipConfig {
    pub font_size: f32,
    pub color: Color,
    pub background: Color,
    pub padding: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for TooltipConfig {
    fn default() -> Self {
        Self {
            font_size: 12.0,
            color: Color::WHITE,
            background: Color::srgba(0.1, 0.1, 0.1, 0.9),
            padding: 4.0,
            offset_x: 15.0,
            offset_y: -10.0,
        }
    }
}

impl TooltipConfig {
    pub fn load(mods: &ModRegistry) -> Self {
        let mut cfg = Self::default();
        let Some(content) = mods.load_data("panel_tooltip.toml") else { return cfg; };
        let Ok(val) = toml::from_str::<toml::Value>(&content) else { return cfg; };
        if let Some(v) = val.get("font_size").and_then(|v| v.as_float()) { cfg.font_size = v as f32; }
        if let Some(s) = val.get("color").and_then(|v| v.as_str()) { cfg.color = parse_color(s); }
        if let Some(s) = val.get("background").and_then(|v| v.as_str()) { cfg.background = parse_color(s); }
        if let Some(v) = val.get("padding").and_then(|v| v.as_float()) { cfg.padding = v as f32; }
        if let Some(v) = val.get("offset_x").and_then(|v| v.as_float()) { cfg.offset_x = v as f32; }
        if let Some(v) = val.get("offset_y").and_then(|v| v.as_float()) { cfg.offset_y = v as f32; }
        cfg
    }
}

// ── Shared types ──

#[derive(Resource, Default)]
pub struct TooltipText(pub Option<String>);

#[derive(Component)]
pub struct TooltipMarker;

// ── System ──

pub fn tooltip_ui(
    tooltip: Res<TooltipText>,
    config: Res<TooltipConfig>,
    windows: Query<&Window>,
    mut text_query: Query<(Entity, &mut Text, &mut Node), With<TooltipMarker>>,
    mut commands: Commands,
) {
    if let Some(ref msg) = tooltip.0 {
        if let Ok(window) = windows.single() {
            if let Ok((_, mut text, mut style)) = text_query.single_mut() {
                **text = msg.clone();
                style.display = Display::Flex;
                if let Some(cursor) = window.cursor_position() {
                    style.left = Val::Px(cursor.x + config.offset_x);
                    style.top = Val::Px(cursor.y + config.offset_y);
                }
            } else {
                commands.spawn((
                    TooltipMarker,
                    Text::new(msg.as_str()),
                    tf(config.font_size),
                    TextColor(config.color),
                    Node {
                        position_type: PositionType::Absolute,
                        display: Display::None,
                        padding: UiRect::all(Val::Px(config.padding)),
                        ..default()
                    },
                    BackgroundColor(config.background),
                ));
            }
        }
    } else if let Ok((_, _, mut style)) = text_query.single_mut() {
        style.display = Display::None;
    }
}

fn parse_color(s: &str) -> Color {
    if s.starts_with('#') || s.contains("0x") {
        let hex = s.trim_start_matches('#').trim_start_matches("0x");
        if let Ok(r) = u8::from_str_radix(&hex[0..2], 16) {
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            return Color::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
        }
    }
    let parts: Vec<f32> = s.split(',').filter_map(|p| p.trim().parse::<f32>().ok()).collect();
    if parts.len() >= 4 {
        Color::srgba(parts[0], parts[1], parts[2], parts[3])
    } else if parts.len() >= 3 {
        Color::srgb(parts[0], parts[1], parts[2])
    } else {
        Color::srgba(0.1, 0.1, 0.1, 0.9)
    }
}
