use bevy::prelude::*;

use crate::core::game_font::tf;
use crate::core::modding::ModRegistry;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{Active, Building, PowerConsumer, PowerProducer};
use crate::economy::resource::Inventory;
use crate::economy::spatial::SpatialRegistry;
use crate::map::components::{TilePosition, cursor_to_tile};
use crate::map::config::MapConfig;
use crate::rendering::minimap::MinimapCamera;

#[derive(Component)]
pub struct BuildingTooltip;

#[derive(Resource, Default)]
pub struct TooltipTarget(pub Option<Entity>);

#[derive(Resource)]
pub struct BuildingTooltipConfig {
    pub font_size: f32,
    pub color: Color,
    pub background: Color,
    pub padding: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub z_index: i32,
}

fn parse_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::srgb(0.5, 0.5, 0.5);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
    Color::srgb(r, g, b)
}

impl BuildingTooltipConfig {
    pub fn load(mods: &ModRegistry) -> Self {
        let content = mods.load_data("panel_building_tooltip.toml").unwrap_or_default();
        let Ok(config) = toml::from_str::<toml::Value>(&content) else {
            return Self::default();
        };
        let bg_hex = config.get("background").and_then(|v| v.as_str()).unwrap_or("#000000");
        let bg = parse_hex(bg_hex);
        let bg_srgba = bg.to_srgba();
        let opacity = config.get("background_opacity").and_then(|v| v.as_float()).unwrap_or(0.75) as f32;
        Self {
            font_size: config
                .get("font_size")
                .and_then(|v| v.as_float())
                .unwrap_or(11.0) as f32,
            color: config
                .get("color")
                .and_then(|v| v.as_str())
                .map(parse_hex)
                .unwrap_or(Color::srgb(0.9, 0.9, 0.9)),
            background: Color::srgba(bg_srgba.red, bg_srgba.green, bg_srgba.blue, opacity),
            padding: config
                .get("padding")
                .and_then(|v| v.as_float())
                .unwrap_or(4.0) as f32,
            offset_x: config
                .get("offset_x")
                .and_then(|v| v.as_float())
                .unwrap_or(8.0) as f32,
            offset_y: config
                .get("offset_y")
                .and_then(|v| v.as_float())
                .unwrap_or(8.0) as f32,
            z_index: config
                .get("z_index")
                .and_then(|v| v.as_integer())
                .unwrap_or(200) as i32,
        }
    }
}

impl Default for BuildingTooltipConfig {
    fn default() -> Self {
        Self {
            font_size: 11.0,
            color: Color::srgb(0.9, 0.9, 0.9),
            background: Color::srgba(0.0, 0.0, 0.0, 0.75),
            padding: 4.0,
            offset_x: 8.0,
            offset_y: 8.0,
            z_index: 200,
        }
    }
}

pub fn building_tooltip_system(
    mut commands: Commands,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<MinimapCamera>)>,
    cfg: Res<MapConfig>,
    spatial: Res<SpatialRegistry>,
    building_q: Query<(
        &Building,
        Option<&Active>,
        Option<&Inventory>,
        Option<&PowerConsumer>,
        Option<&PowerProducer>,
    )>,
    registry: Res<BuildingRegistry>,
    mut target: ResMut<TooltipTarget>,
    tooltip_q: Query<Entity, With<BuildingTooltip>>,
    config: Res<BuildingTooltipConfig>,
) {
    let Some(TilePosition { x, y }) = cursor_to_tile(&windows, &camera, &cfg) else {
        clear_tooltip(&mut commands, &tooltip_q, &mut target);
        return;
    };
    let Some(entity) = spatial.at(x, y) else {
        clear_tooltip(&mut commands, &tooltip_q, &mut target);
        return;
    };
    let Ok((building, active, inventory, power_consumer, power_producer)) = building_q.get(entity)
    else {
        clear_tooltip(&mut commands, &tooltip_q, &mut target);
        return;
    };

    if target.0 == Some(entity) {
        return;
    }

    clear_tooltip(&mut commands, &tooltip_q, &mut target);
    target.0 = Some(entity);

    let def = registry.get(&building.kind);
    let name = def.map(|d| d.name.as_str()).unwrap_or(&building.kind);
    let mut lines: Vec<String> = Vec::new();

    lines.push(format!("┌─ {} ─────────────────┐", shorten(name, 22)));

    if let Some(a) = active {
        let state = if a.0 { "▶ Actif" } else { "⏸ En pause" };
        lines.push(format!("│ {}                      │", state));
    }

    if let Some(inv) = inventory {
        let count = inv.total();
        if count > 0 {
            let items: Vec<String> = inv
                .iter_occupied()
                .take(2)
                .map(|(r, a)| format!("{} x{}", r.display_name(), a))
                .collect();
            let joined = items.join(", ");
            lines.push(format!("│ {}                      │", shorten(&joined, 30)));
        }
    }

    let mut power_parts: Vec<String> = Vec::new();
    if let Some(pc) = power_consumer {
        let status = if pc.satisfied { "✓" } else { "✗" };
        power_parts.push(format!("⚡{:.0}W{}", pc.draw, status));
    }
    if let Some(pp) = power_producer {
        power_parts.push(format!("🔋{:.0}W", pp.output));
    }
    if !power_parts.is_empty() {
        lines.push(format!("│ {}                  │", power_parts.join(" ")));
    }

    lines.push("└────────────────────────────┘".to_string());

    let full_text = lines.join("\n");
    commands.spawn((
        BuildingTooltip,
        Text::new(full_text),
        tf(config.font_size),
        TextColor(config.color),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(config.offset_y),
            left: Val::Px(config.offset_x),
            padding: UiRect::all(Val::Px(config.padding)),
            ..default()
        },
        BackgroundColor(config.background),
        ZIndex(config.z_index),
    ));
}

fn clear_tooltip(
    commands: &mut Commands,
    tooltip_q: &Query<Entity, With<BuildingTooltip>>,
    target: &mut TooltipTarget,
) {
    for entity in tooltip_q.iter() {
        commands.entity(entity).despawn();
    }
    target.0 = None;
}

fn shorten(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
