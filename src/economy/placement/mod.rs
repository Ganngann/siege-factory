pub mod build;
pub use build::*;
pub mod belt_drag;
pub use belt_drag::*;
pub mod deconstruct;
pub use deconstruct::*;

use crate::core::input::KeyBindings;
use crate::economy::belt::BeltSlots;
use crate::economy::belt::compute_slot_positions;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{
    BeltDirection, BeltDrag, BuildMode, DeconstructDrag, DeconstructMode, Ghost, UiIsBlocking,
};
use crate::economy::components::{Building, OccupiedTiles};
use crate::economy::spatial::SpatialRegistry;
use crate::events::DeconstructAreaEvent;
use crate::map::components::{HoveredTile, TilePosition, cursor_to_tile};
use crate::map::config::MapConfig;
use crate::rendering::minimap::MinimapCamera;
use crate::rendering::{PreviewMaterials, ShapeCache};
use bevy::prelude::*;

// SUGGEST: extraire dans un struct SystemParam (clippy::too_many_arguments)
pub fn build_mode_input(
    mut build_mode: ResMut<BuildMode>,
    mut deconstruct: ResMut<DeconstructMode>,
    mut belt_dir: ResMut<BeltDirection>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
    cfg: Res<MapConfig>,
    mut placed_belts: Query<(&mut BeltSlots, &TilePosition)>,
    hovered: Res<HoveredTile>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if bindings.just_pressed("build_rotate", &keys, &mouse)
        && build_mode.0.as_deref() == Some("belt")
    {
        if let Some(pos) = hovered.0 {
            let mut rotated = false;
            for (mut belt, tile_pos) in placed_belts.iter_mut() {
                if tile_pos.x == pos.x && tile_pos.y == pos.y {
                    belt.direction = belt.direction.next();
                    belt.slot_positions = compute_slot_positions(
                        tile_pos.x,
                        tile_pos.y,
                        belt.direction,
                        belt.items.len() as u32,
                        cfg.tile_size,
                    );
                    rotated = true;
                    break;
                }
            }
            if !rotated {
                belt_dir.0 = belt_dir.0.next();
            }
        } else {
            belt_dir.0 = belt_dir.0.next();
        }
    }

    if bindings.just_pressed("cancel_build", &keys, &buttons) {
        build_mode.0 = None;
        deconstruct.0 = false;
    }
}

// ── Belt click/drag ──

#[allow(clippy::too_many_arguments)]
pub fn track_belt_drag(
    mut commands: Commands,
    mut drag: ResMut<BeltDrag>,
    build_mode: Res<BuildMode>,
    cfg: Res<MapConfig>,
    spatial: Res<SpatialRegistry>,
    windows: Query<&Window>,
    // SUGGEST: type CameraQuery = Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<MinimapCamera>)> (clippy::type_complexity)
    camera: Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<MinimapCamera>)>,
    producers: Query<&TilePosition, With<crate::economy::components::Miner>>,
    belt_read: Query<(&TilePosition, &BeltSlots)>,
    buttons: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
    registry: Res<BuildingRegistry>,
    mut toast_queue: ResMut<crate::core::toast::ToastQueue>,
    ui_blocking: Res<UiIsBlocking>,
) {
    if ui_blocking.0 {
        return;
    }
    let Some(ref kind) = build_mode.0 else {
        drag.start_coord = None;
        return;
    };
    let Some(def) = registry.get(kind) else {
        drag.start_coord = None;
        return;
    };
    if def.belt.is_none() && !def.drag_placement {
        drag.start_coord = None;
        return;
    }

    let Some(TilePosition { x: tx, y: ty }) = cursor_to_tile(&windows, &camera, &cfg) else {
        return;
    };

    if buttons.just_pressed(bindings.mouse("place")) {
        let has_belt = belt_read.iter().any(|(pos, _)| pos.x == tx && pos.y == ty);
        let is_free = spatial.is_free(tx, ty);
        if has_belt || is_free {
            drag.start_coord = Some((tx, ty));
        } else {
            toast_queue.0.push("Tile occupied".to_string());
        }
        return;
    }

    if buttons.just_released(bindings.mouse("place")) {
        let Some(start) = drag.start_coord.take() else {
            return;
        };

        let belt_data: Vec<((i32, i32), crate::economy::components::Direction)> = belt_read
            .iter()
            .map(|(pos, bs)| ((pos.x, pos.y), bs.direction))
            .collect();

        let line = build::compute_line(start, (tx, ty));
        let single = line.len() == 1;

        let mut existing: Vec<(i32, i32, crate::economy::components::Direction)> = Vec::new();
        let mut new_tiles: Vec<(i32, i32, crate::economy::components::Direction)> = Vec::new();

        for &(bx, by, base_dir) in &line {
            let dir = if single {
                build::auto_detect_direction_from_data(
                    bx,
                    by,
                    &producers,
                    &belt_data,
                    crate::economy::components::Direction::East,
                )
            } else {
                base_dir
            };
            let has_belt = belt_data.iter().any(|&((px, py), _)| px == bx && py == by);
            if has_belt {
                existing.push((bx, by, dir));
            } else {
                new_tiles.push((bx, by, dir));
            }
        }

        if existing.is_empty() && new_tiles.is_empty() {
            toast_queue.0.push("No valid tiles".to_string());
            return;
        }

        commands.trigger(crate::events::BeltDragCompleted {
            kind: kind.clone(),
            new_tiles,
            existing,
        });
    }
}

/// Preview the deconstruct drag zone as a red ghost overlay of actual buildings
// SUGGEST: extraire dans un struct SystemParam (clippy::too_many_arguments)
pub fn deconstruct_drag_preview(
    mut commands: Commands,
    deconstruct: Res<DeconstructMode>,
    deconstruct_drag: Res<DeconstructDrag>,
    cfg: Res<MapConfig>,
    shapes: Res<ShapeCache>,
    preview_materials: Res<PreviewMaterials>,
    hovered: Res<HoveredTile>,
    spatial: Res<SpatialRegistry>,
    building_query: Query<(&Building, &TilePosition, &OccupiedTiles)>,
    registry: Res<BuildingRegistry>,
) {
    if !deconstruct.0 {
        return;
    }
    let Some((sx, sy)) = deconstruct_drag.start_coord else {
        return;
    };
    let Some(TilePosition { x: tx, y: ty }) = hovered.0 else {
        return;
    };

    let x1 = sx.min(tx);
    let x2 = sx.max(tx);
    let y1 = sy.min(ty);
    let y2 = sy.max(ty);

    let entities = spatial.entities_in_rect(x1, y1, x2, y2);

    for entity in entities {
        let Ok((building, pos, _tiles)) = building_query.get(entity) else {
            continue;
        };
        let Some(def) = registry.get(&building.kind) else {
            continue;
        };
        let (tw, th) = def.tile_size;
        let cx = (pos.x as f32 + (tw as f32 - 1.0) * 0.5) * cfg.tile_size;
        let cy = (pos.y as f32 + (th as f32 - 1.0) * 0.5) * cfg.tile_size;
        let mesh = shapes.get_visual(&def.visual);
        commands.spawn((
            Ghost,
            Mesh2d(mesh),
            MeshMaterial2d(preview_materials.deconstruct_building.clone()),
            Transform::from_xyz(cx, cy, 10.0),
        ));
    }

    // Single rectangle ghost covering the entire zone (replaces per-tile grid)
    let ts = cfg.tile_size;
    let mesh_size = ts - 4.0; // ShapeCache square size
    let n_x = (x2 - x1 + 1) as f32;
    let n_y = (y2 - y1 + 1) as f32;
    let zone_cx = (x1 + x2) as f32 * 0.5 * ts;
    let zone_cy = (y1 + y2) as f32 * 0.5 * ts;
    commands.spawn((
        Ghost,
        Mesh2d(shapes.square.clone()),
        MeshMaterial2d(preview_materials.deconstruct_zone.clone()),
        Transform::from_xyz(zone_cx, zone_cy, 9.9).with_scale(Vec3::new(
            n_x * ts / mesh_size,
            n_y * ts / mesh_size,
            1.0,
        )),
    ));
}

// ── Deconstruct drag ──

// SUGGEST: extraire dans un struct SystemParam (clippy::too_many_arguments)
pub fn track_deconstruct_drag(
    mut commands: Commands,
    mut drag: ResMut<DeconstructDrag>,
    deconstruct: Res<DeconstructMode>,
    cfg: Res<MapConfig>,
    windows: Query<&Window>,
    // SUGGEST: type CameraQuery = Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<MinimapCamera>)> (clippy::type_complexity)
    camera: Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<MinimapCamera>)>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
    ui_blocking: Res<UiIsBlocking>,
) {
    if ui_blocking.0 {
        return;
    }
    if !deconstruct.0 {
        drag.start_coord = None;
        return;
    }

    let Some(TilePosition { x: tx, y: ty }) = cursor_to_tile(&windows, &camera, &cfg) else {
        return;
    };

    if bindings.just_pressed("place", &keys, &buttons) && drag.start_coord.is_none() {
        drag.start_coord = Some((tx, ty));
        return;
    }

    if buttons.just_released(bindings.mouse("place")) {
        let Some(start) = drag.start_coord.take() else {
            return;
        };
        commands.trigger(DeconstructAreaEvent {
            start: TilePosition {
                x: start.0,
                y: start.1,
            },
            end: TilePosition { x: tx, y: ty },
        });
    }
}
