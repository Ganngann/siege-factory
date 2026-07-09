use bevy::prelude::*;

use crate::core::game_font::tf;

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

/// Affiche un tooltip au survol d'un bâtiment : nom, état, recette, power.
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

    // Same target — no update needed
    if target.0 == Some(entity) {
        return;
    }

    // Clear old tooltip
    clear_tooltip(&mut commands, &tooltip_q, &mut target);
    target.0 = Some(entity);

    // Build tooltip text
    let def = registry.get(&building.kind);
    let name = def.map(|d| d.name.as_str()).unwrap_or(&building.kind);
    let mut lines: Vec<String> = Vec::new();

    lines.push(format!("┌─ {} ─────────────────┐", shorten(name, 22)));

    // State
    if let Some(a) = active {
        let state = if a.0 { "▶ Actif" } else { "⏸ En pause" };
        lines.push(format!("│ {}                      │", state));
    }

    // Inventory
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

    // Power
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

    // Spawn tooltip UI
    let full_text = lines.join("\n");
    commands.spawn((
        BuildingTooltip,
        Text::new(full_text),
        tf(11.0),
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(8.0),
            padding: UiRect::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
        ZIndex(200),
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
