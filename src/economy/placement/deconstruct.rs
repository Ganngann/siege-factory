use crate::core::input::KeyBindings;
use crate::core::utils::silent_despawn;
use crate::core::toast::ToastQueue;
use crate::economy::belt::BeltSlots;
use crate::economy::building::{BuildingDef, BuildingRegistry};
use crate::economy::components::{Building, DeconstructMode, UiIsBlocking};
use crate::economy::game_components::Player;
use crate::economy::resource::Inventory;
use crate::economy::spatial::SpatialRegistry;
use crate::events::DeconstructAreaEvent;
use crate::map::components::{TilePosition, cursor_to_tile};
use crate::map::config::MapConfig;
use crate::rendering::minimap::MinimapCamera;
use bevy::prelude::*;

// SUGGEST: extraire dans un struct SystemParam (clippy::too_many_arguments)
pub fn handle_deconstruct_click_v2(
    mut commands: Commands,
    deconstruct: Res<DeconstructMode>,
    cfg: Res<MapConfig>,
    spatial: Res<SpatialRegistry>,
    windows: Query<&Window>,
    // SUGGEST: type CameraQuery = Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<MinimapCamera>)> (clippy::type_complexity)
    camera: Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<MinimapCamera>)>,
    building_query: Query<(&Building, &TilePosition)>,
    mut player_query: Query<&mut Inventory, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
    registry: Res<BuildingRegistry>,
    mut toast_queue: ResMut<ToastQueue>,
    belt_slots_query: Query<&BeltSlots>,
    ui_blocking: Res<UiIsBlocking>,
) {
    if ui_blocking.0 {
        return;
    }
    if !deconstruct.0 {
        return;
    }
    if !bindings.just_pressed("place", &keys, &buttons) {
        return;
    }

    let Some(TilePosition { x: tx, y: ty }) = cursor_to_tile(&windows, &camera, &cfg) else {
        return;
    };

    let Some(entity) = spatial.at(tx, ty) else {
        return;
    };
    let Ok((building, _)) = building_query.get(entity) else {
        return;
    };

    let def = match registry.get(&building.kind) {
        Some(d) => d,
        None => return,
    };

    if !def.can_deconstruct {
        toast_queue
            .0
            .push(format!("Cannot deconstruct {}", building.name));
        return;
    }

    let refund_names = if let Ok(mut player_inv) = player_query.single_mut() {
        deconstruct_entity(
            def,
            &mut player_inv,
            &mut commands,
            &belt_slots_query,
            entity,
        )
    } else {
        return;
    };
    toast_queue.0.push(format!(
        "{} dismantled (+{})",
        building.name,
        refund_names.join(", ")
    ));
}

/// Refund resources, despawn belt sprites, and despawn the entity.
/// Returns per-resource refund strings for toast messages.
pub fn deconstruct_entity(
    def: &BuildingDef,
    player_inv: &mut Inventory,
    commands: &mut Commands,
    belt_slots_query: &Query<&BeltSlots>,
    entity: Entity,
) -> Vec<String> {
    let mut refund_names = Vec::new();
    for c in &def.cost {
        let refund = (c.amount as f32 * def.refund_ratio).ceil() as u32;
        if refund > 0 {
            player_inv.add(&c.resource, refund);
            refund_names.push(format!("{} {}", refund, c.resource.display_name()));
        }
    }
    if let Ok(belt_slots) = belt_slots_query.get(entity) {
        for sprite_entity in belt_slots.slot_sprites.iter().flatten() {
            silent_despawn(commands, *sprite_entity);
        }
    }
    silent_despawn(commands, entity);
    refund_names
}

/// Observer for `DeconstructAreaEvent`. Despawns all buildings in the zone.
// SUGGEST: extraire dans un struct SystemParam (clippy::too_many_arguments)
pub fn on_deconstruct_area(
    on: On<DeconstructAreaEvent>,
    mut commands: Commands,
    spatial: Res<SpatialRegistry>,
    building_query: Query<(&Building, &crate::economy::components::OccupiedTiles)>,
    belt_slots_query: Query<&BeltSlots>,
    mut player_query: Query<&mut Inventory, With<Player>>,
    registry: Res<BuildingRegistry>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    let ev = on.event();
    let x1 = ev.start.x.min(ev.end.x);
    let x2 = ev.start.x.max(ev.end.x);
    let y1 = ev.start.y.min(ev.end.y);
    let y2 = ev.start.y.max(ev.end.y);

    let entities = spatial.entities_in_rect(x1, y1, x2, y2);
    let mut count = 0u32;
    let mut refund_names: Vec<String> = Vec::new();

    if let Ok(mut player_inv) = player_query.single_mut() {
        for entity in entities {
            let Ok((building, _tiles)) = building_query.get(entity) else {
                continue;
            };

            if let Some(def) = registry.get(&building.kind) {
                if !def.can_deconstruct {
                    continue;
                }
                let mut names = deconstruct_entity(
                    def,
                    &mut player_inv,
                    &mut commands,
                    &belt_slots_query,
                    entity,
                );
                refund_names.append(&mut names);
            }
            count += 1;
        }
    }

    if count > 0 {
        toast_queue.0.push(format!(
            "Zone deconstruct: {} building(s) removed (+{})",
            count,
            refund_names.join(", ")
        ));
    }
}
