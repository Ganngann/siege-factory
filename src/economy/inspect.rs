use bevy::prelude::*;
use crate::core::toast::ToastQueue;
use crate::economy::components::{
    Building, BuildMode, DeconstructMode, OccupiedTiles, Sorter, BuildingPopup,
};
use crate::economy::belt::BeltSlots;
use crate::economy::building::BuildingRegistry;
use crate::economy::resource::{ResourceRegistry, Inventory};
use crate::enemy::components::Health;
use crate::map::config::MapConfig;

/// Click on a building (outside build/deconstruct) → show floating popup
pub fn building_inspect_click(
    mut commands: Commands,
    mut popup: ResMut<BuildingPopup>,
    build_mode: Res<BuildMode>,
    deconstruct: Res<DeconstructMode>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    cfg: Res<MapConfig>,
    building_query: Query<(Entity, &OccupiedTiles, &Building)>,
    inventory_query: Query<&Inventory>,
    health_query: Query<&Health>,
    belt_query: Query<&BeltSlots>,
    registry: Res<BuildingRegistry>,
    resource_registry: Res<ResourceRegistry>,
) {
    // Escape → dismiss popup
    if keys.just_pressed(KeyCode::Escape) {
        if let Some(entity) = popup.0.take() {
            commands.entity(entity).despawn();
        }
        return;
    }

    if build_mode.0.is_some() || deconstruct.0 { return; }
    if !buttons.just_pressed(MouseButton::Left) { return; }

    // If popup is already open, despawn and re-evaluate
    if let Some(entity) = popup.0.take() {
        commands.entity(entity).despawn();
    }

    let tile_size = cfg.tile_size;
    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_transform)) = camera.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else { return };

    let tile_x = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let tile_y = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

    if tile_x < 0 || tile_y < 0 { return; }
    let tx = tile_x as u32;
    let ty = tile_y as u32;

    let Some((entity, _, building)) = building_query.iter().find(|(_, tiles, _)|
        tiles.0.iter().any(|&(x, y)| x == tx && y == ty)
    ) else { return };

    // Build popup text
    let mut lines = Vec::new();
    lines.push(format!("=== {} ===", building.name));

    if let Some(def) = registry.get(&building.kind) {
        lines.push(format!("Kind: {}", def.id));
    }

    if let Ok(health) = health_query.get(entity) {
        lines.push(format!("HP: {}/{}", health.current, health.max));

        // Repair info
        if health.current < health.max {
            if let Some(def) = registry.get(&building.kind) {
                let max_hp = health.max as f32;
                let missing = max_hp - health.current as f32;
                let base_cost: u32 = def.cost.iter().map(|c| c.amount).sum();
                let repair_cost = (def.repair_cost_ratio * base_cost as f32 * missing / max_hp).ceil() as u32;
                lines.push(format!("Repair cost: {} Ore", repair_cost));
            }
        }
    }

    if let Ok(inv) = inventory_query.get(entity) {
        if inv.total() > 0 {
            for (res_id, amount) in &inv.resources {
                let def = resource_registry.get(*res_id);
                let cap = if inv.capacity > 0 {
                    format!(" / {}", inv.capacity)
                } else { String::new() };
                lines.push(format!("  {}: {}{}", def.name, amount, cap));
            }
        }
        if inv.capacity > 0 {
            lines.push(format!("Capacity: {}/{}", inv.total(), inv.capacity));
        }
    }

    if let Ok(bs) = belt_query.get(entity) {
        let occupied_slots = bs.slots.iter().filter(|s| s.is_some()).count();
        if occupied_slots > 0 {
            lines.push(format!("Items in transit: {}/{}", occupied_slots, bs.slots.len()));
        }
    }

    if lines.len() <= 1 { return; } // Nothing interesting

    let text = lines.join("\n");

    // Spawn popup as a UI text element
    let root = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(80.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
        Outline {
            width: Val::Px(1.0),
            offset: Val::ZERO,
            color: Color::WHITE,
        },
        BuildingPopupMarker,
    )).id();

    commands.spawn((
        Text::new(text),
        TextFont::from_font_size(14.0),
        TextColor(Color::WHITE),
        Node {
            max_width: Val::Px(250.0),
            ..default()
        },
        BuildingPopupMarker,
    )).set_parent_in_place(root);

    popup.0 = Some(root);
}

/// Click on a sorter (not in build/deconstruct) → toggle invert mode
pub fn sorter_toggle_click(
    build_mode: Res<BuildMode>,
    deconstruct: Res<DeconstructMode>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    cfg: Res<MapConfig>,
    building_query: Query<(Entity, &OccupiedTiles, &Building)>,
    mut sorter_query: Query<&mut Sorter>,
    mut toast_queue: ResMut<ToastQueue>,
    popup: Res<BuildingPopup>,
) {
    if build_mode.0.is_some() || deconstruct.0 { return; }
    if !buttons.just_pressed(MouseButton::Left) { return; }
    if popup.0.is_some() { return; } // Let the popup system handle it

    let tile_size = cfg.tile_size;
    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_transform)) = camera.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else { return };

    let tile_x = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let tile_y = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

    if tile_x < 0 || tile_y < 0 { return; }
    let tx = tile_x as u32;
    let ty = tile_y as u32;

    let Some((entity, _, building)) = building_query.iter().find(|(_, tiles, _)|
        tiles.0.iter().any(|&(x, y)| x == tx && y == ty)
    ) else { return };

    if building.kind != "sorter" { return; }

    if let Ok(mut sorter) = sorter_query.get_mut(entity) {
        sorter.inverted = !sorter.inverted;
        let mode = if sorter.inverted {
            "filtered → straight, others → side"
        } else {
            "filtered → side, others → straight"
        };
        toast_queue.0.push(format!("Sorter toggled: {}", mode));
    }
}

/// Marker component for building popup UI entities
#[derive(Component)]
pub struct BuildingPopupMarker;

/// Cleanup popup on state exit
pub fn cleanup_popup(mut commands: Commands, query: Query<Entity, With<BuildingPopupMarker>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
