use crate::agriculture::components::{Crop, Farm};
use crate::core::input::KeyBindings;
use crate::core::toast::ToastQueue;
use crate::economy::belt::{BeltSlots, compute_slot_positions};
use crate::economy::building::BuildingRegistry;
use crate::economy::resource::Cost;
use crate::economy::components::{
    Active, Archive, Assembler, BeltDirection, BeltDrag, BuildMode, BuildPreview, Building,
    DeconstructDrag, DeconstructMode, Direction, DiscoveredRecipes, Ghost, Miner, OccupiedTiles,
    Player, PowerConsumer, PowerPole, PowerProducer, ProductionCounter, ResourceDeposit, Sorter,
    Splitter, Storage, TurretCombat, UiIsBlocking, UnbuiltBuilding,
};

use crate::economy::resource::{Inventory, ResourceId};
use crate::economy::spatial::SpatialRegistry;
use crate::events::{BeltDragCompleted, DeconstructAreaEvent, DespawnDeposit};
use crate::map::components::{HoveredTile, TilePosition};
use crate::map::config::MapConfig;
use crate::map::tile_grid::{CHUNK_SIZE, ChunkGrid};
use crate::rendering::{PreviewMaterials, ShapeCache, direction_arrow};
use bevy::prelude::*;

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
    if bindings.just_pressed("build_rotate", &keys, &mouse) && build_mode.0.as_deref() == Some("belt") {
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

// ── Auto-direction ──

fn detect_producer_direction(
    tx: i32,
    ty: i32,
    producers: &Query<&TilePosition, With<Miner>>,
) -> Option<Direction> {
    let offsets = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let dirs = [
        Direction::East,
        Direction::North,
        Direction::West,
        Direction::South,
    ];
    for (&(dx, dy), &dir) in offsets.iter().zip(dirs.iter()) {
        let nx = tx + dx;
        let ny = ty + dy;
        if producers.iter().any(|pos| pos.x == nx && pos.y == ny) {
            return Some(dir);
        }
    }
    None
}

fn auto_detect_direction(
    tx: i32,
    ty: i32,
    producers: &Query<&TilePosition, With<Miner>>,
    belts_query: &Query<(&TilePosition, &BeltSlots)>,
    default: Direction,
) -> Direction {
    if let Some(dir) = detect_producer_direction(tx, ty, producers) {
        return dir;
    }

    for (pos, slots) in belts_query.iter() {
        let (odx, ody) = slots.direction.offset();
        if pos.x + odx == tx && pos.y + ody == ty {
            return slots.direction;
        }
    }

    default
}

fn auto_detect_direction_from_data(
    tx: i32,
    ty: i32,
    producers: &Query<&TilePosition, With<Miner>>,
    belt_data: &[((i32, i32), Direction)],
    default: Direction,
) -> Direction {
    if let Some(dir) = detect_producer_direction(tx, ty, producers) {
        return dir;
    }

    for &((px, py), dir) in belt_data {
        let (odx, ody) = dir.offset();
        if px + odx == tx && py + ody == ty {
            return dir;
        }
    }

    default
}

// ── Click-drag helpers ──

fn compute_line(start: (i32, i32), end: (i32, i32)) -> Vec<(i32, i32, Direction)> {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let adx = dx.abs();
    let ady = dy.abs();

    if adx == 0 && ady == 0 {
        return vec![(start.0, start.1, Direction::East)];
    }

    let mut result = Vec::new();

    if adx > 0 && ady > 0 {
        let sdx = dx.signum();
        let sdy = dy.signum();
        let dir_x = if sdx > 0 {
            Direction::East
        } else {
            Direction::West
        };
        let dir_y = if sdy > 0 {
            Direction::North
        } else {
            Direction::South
        };

        if adx >= ady {
            for i in 0..adx {
                result.push((start.0 + sdx * i, start.1, dir_x));
            }
            for i in 0..=ady {
                result.push((end.0, start.1 + sdy * i, dir_y));
            }
        } else {
            for i in 0..ady {
                result.push((start.0, start.1 + sdy * i, dir_y));
            }
            for i in 0..=adx {
                result.push((start.0 + sdx * i, end.1, dir_x));
            }
        }
    } else if adx > 0 {
        let sdx = dx.signum();
        let dir = if sdx > 0 {
            Direction::East
        } else {
            Direction::West
        };
        for i in 0..=adx {
            result.push((start.0 + sdx * i, start.1, dir));
        }
    } else {
        let sdy = dy.signum();
        let dir = if sdy > 0 {
            Direction::North
        } else {
            Direction::South
        };
        for i in 0..=ady {
            result.push((start.0, start.1 + sdy * i, dir));
        }
    }

    result
}

// ── Multi-tile helpers ──

fn compute_footprint(tx: i32, ty: i32, tw: u32, th: u32) -> Vec<(i32, i32)> {
    let mut tiles = Vec::with_capacity((tw * th) as usize);
    for dx in 0..tw {
        for dy in 0..th {
            tiles.push((tx + dx as i32, ty + dy as i32));
        }
    }
    tiles
}

// ── Deconstruction ──

pub fn handle_deconstruct_click_v2(
    mut commands: Commands,
    deconstruct: Res<DeconstructMode>,
    cfg: Res<MapConfig>,
    spatial: Res<SpatialRegistry>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
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

    let tile_size = cfg.tile_size;
    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_transform)) = camera.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else {
        return;
    };

    let tx = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let ty = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

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

    let mut refund_names = Vec::new();
    for c in &def.cost {
        let refund = (c.amount as f32 * def.refund_ratio).ceil() as u32;
        if refund > 0 {
            if let Ok(mut player_inv) = player_query.single_mut() {
                player_inv.add(&c.resource, refund);
            }
            refund_names.push(format!("{} {}", refund, c.resource.display_name()));
        }
    }

    if let Ok(belt_slots) = belt_slots_query.get(entity) {
        for sprite_entity in belt_slots.slot_sprites.iter().flatten() {
            commands.entity(*sprite_entity).despawn();
        }
    }

    commands.entity(entity).despawn();
    toast_queue.0.push(format!(
        "{} dismantled (+{})",
        building.name,
        refund_names.join(", ")
    ));
}

// ── Preview ──

#[allow(clippy::too_many_arguments)]
pub fn update_build_preview(
    mut commands: Commands,
    mut preview: ResMut<BuildPreview>,
    build_mode: Res<BuildMode>,
    belt_dir: Res<BeltDirection>,
    deconstruct: Res<DeconstructMode>,
    cfg: Res<MapConfig>,
    shapes: Res<ShapeCache>,
    preview_materials: Res<PreviewMaterials>,
    spatial: Res<SpatialRegistry>,
    deposits: Query<&TilePosition, With<ResourceDeposit>>,
    producers: Query<&TilePosition, With<Miner>>,
    belts_query: Query<(&TilePosition, &BeltSlots)>,
    registry: Res<BuildingRegistry>,
    hovered: Res<HoveredTile>,
    ghosts: Query<Entity, With<Ghost>>,
    drag: Res<BeltDrag>,
) {
    for entity in ghosts.iter() {
        commands.entity(entity).despawn();
    }
    preview.0 = None;

    if deconstruct.0 {
        let Some(TilePosition { x: tx, y: ty }) = hovered.0 else {
            return;
        };
        let occupied_here = !spatial.is_free(tx, ty);
        let mat = if occupied_here {
            preview_materials.deconstruct_building.clone()
        } else {
            preview_materials.deconstruct_zone.clone()
        };
        commands.spawn((
            Ghost,
            Mesh2d(shapes.rectangle.clone()),
            MeshMaterial2d(mat),
            Transform::from_xyz(tx as f32 * cfg.tile_size, ty as f32 * cfg.tile_size, 1.8),
        ));
        return;
    }

    let Some(ref kind) = build_mode.0 else { return };
    let Some(def) = registry.get(kind) else {
        return;
    };
    let Some(TilePosition { x: tx, y: ty }) = hovered.0 else {
        return;
    };

    // ── Drag line preview ──
    if def.belt.is_some() || def.drag_placement {
        if let Some((sx, sy)) = drag.start_coord {
            let line = compute_line((sx, sy), (tx, ty));
            for &(lx, ly, dir) in &line {
                let has_belt = belts_query.iter().any(|(p, _)| p.x == lx && p.y == ly);
                let valid = has_belt || spatial.is_free(lx, ly);
                let mat_handle = if valid {
                    preview_materials.build_valid.clone()
                } else {
                    preview_materials.build_invalid.clone()
                };
                let cx = lx as f32 * cfg.tile_size;
                let cy = ly as f32 * cfg.tile_size;
                if def.belt.is_some() {
                    let angle = match dir {
                        Direction::East => 0.0,
                        Direction::North => std::f32::consts::FRAC_PI_2,
                        Direction::West => std::f32::consts::PI,
                        Direction::South => -std::f32::consts::FRAC_PI_2,
                    };
                    commands.spawn((
                        Ghost,
                        Mesh2d(shapes.rectangle.clone()),
                        MeshMaterial2d(mat_handle),
                        Transform::from_xyz(cx, cy, 1.8)
                            .with_rotation(Quat::from_rotation_z(angle)),
                        Text2d::new(direction_arrow(dir).to_string()),
                        TextFont::from_font_size(18.0),
                        TextColor(if valid {
                            Color::srgba(0.0, 0.8, 0.0, 0.6)
                        } else {
                            Color::srgba(0.8, 0.0, 0.0, 0.5)
                        }),
                        TextLayout::justify(Justify::Center),
                    ));
                } else {
                    commands.spawn((
                        Ghost,
                        Mesh2d(shapes.rectangle.clone()),
                        MeshMaterial2d(mat_handle),
                        Transform::from_xyz(cx, cy, 1.8),
                    ));
                }
            }
            return;
        }
    }

    // ── Multi-tile preview ──
    let (tw, th) = def.tile_size;
    let footprint = compute_footprint(tx, ty, tw, th);

    let valid = if def.requires_deposit {
        deposits.iter().any(|pos| pos.x == tx && pos.y == ty) && spatial.tiles_are_free(&footprint)
    } else {
        spatial.tiles_are_free(&footprint)
    };

    let mat_handle = if valid {
        preview_materials.build_valid.clone()
    } else {
        preview_materials.build_invalid.clone()
    };
    let z = 1.8;
    let cx = (tx as f32 + (tw as f32 - 1.0) * 0.5) * cfg.tile_size;
    let cy = (ty as f32 + (th as f32 - 1.0) * 0.5) * cfg.tile_size;

    let entity = if def.belt.is_some() {
        let dir = auto_detect_direction(tx, ty, &producers, &belts_query, belt_dir.0);
        let angle = match dir {
            Direction::East => 0.0,
            Direction::North => std::f32::consts::FRAC_PI_2,
            Direction::West => std::f32::consts::PI,
            Direction::South => -std::f32::consts::FRAC_PI_2,
        };
        let text_color = if valid {
            Color::srgba(0.0, 0.8, 0.0, 0.6)
        } else {
            Color::srgba(0.8, 0.0, 0.0, 0.5)
        };
        let ghost_entity = commands
            .spawn((
                Ghost,
                Mesh2d(shapes.rectangle.clone()),
                MeshMaterial2d(mat_handle),
                Transform::from_xyz(cx, cy, z).with_rotation(Quat::from_rotation_z(angle)),
                Text2d::new(direction_arrow(dir).to_string()),
                TextFont::from_font_size(18.0),
                TextColor(text_color),
                TextLayout::justify(Justify::Center),
            ))
            .id();

        // Connection indicators
        let offsets = [(1, 0), (0, 1), (-1, 0), (0, -1)];
        let dirs = [
            Direction::East,
            Direction::North,
            Direction::West,
            Direction::South,
        ];
        for (&(dx, dy), &check_dir) in offsets.iter().zip(dirs.iter()) {
            let nx = tx + dx;
            let ny = ty + dy;
            let is_input = producers.iter().any(|pos| pos.x == nx && pos.y == ny)
                || belts_query.iter().any(|(pos, slots)| {
                    let (odx, ody) = slots.direction.offset();
                    pos.x + odx == tx && pos.y + ody == ty
                });
            if is_input || check_dir == dir {
                let indicator_mat = if is_input {
                    preview_materials.indicator_input.clone()
                } else {
                    preview_materials.indicator_output.clone()
                };
                let ix = cx + dx as f32 * cfg.tile_size * 0.4;
                let iy = cy + dy as f32 * cfg.tile_size * 0.4;
                commands.spawn((
                    Ghost,
                    Mesh2d(shapes.circle.clone()),
                    MeshMaterial2d(indicator_mat),
                    Transform::from_xyz(ix, iy, z + 0.1).with_scale(Vec3::splat(0.25)),
                ));
            }
        }

        ghost_entity
    } else {
        let mesh = shapes.get_visual(&def.visual);
        commands
            .spawn((
                Ghost,
                Mesh2d(mesh),
                MeshMaterial2d(mat_handle),
                Transform::from_xyz(cx, cy, z),
            ))
            .id()
    };

    preview.0 = Some(entity);
}

/// Preview the deconstruct drag zone as a red ghost overlay of actual buildings
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

fn can_afford(hq_inv: &Inventory, cost: &[Cost]) -> bool {
    cost.iter().all(|c| hq_inv.get(&c.resource) >= c.amount)
}

fn deduct_cost(hq_inv: &mut Inventory, cost: &[Cost]) {
    for c in cost {
        hq_inv.remove(&c.resource, c.amount);
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
    camera: Query<(&Camera, &GlobalTransform)>,
    producers: Query<&TilePosition, With<Miner>>,
    belt_read: Query<(&TilePosition, &BeltSlots)>,
    buttons: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
    registry: Res<BuildingRegistry>,
    mut toast_queue: ResMut<ToastQueue>,
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
    let tile_size = cfg.tile_size;

    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_transform)) = camera.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else {
        return;
    };

    let tx = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let ty = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

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

        let belt_data: Vec<((i32, i32), Direction)> = belt_read
            .iter()
            .map(|(pos, bs)| ((pos.x, pos.y), bs.direction))
            .collect();

        let line = compute_line(start, (tx, ty));
        let single = line.len() == 1;

        let mut existing: Vec<(i32, i32, Direction)> = Vec::new();
        let mut new_tiles: Vec<(i32, i32, Direction)> = Vec::new();

        for &(bx, by, base_dir) in &line {
            let dir = if single {
                auto_detect_direction_from_data(bx, by, &producers, &belt_data, Direction::East)
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

        commands.trigger(BeltDragCompleted {
            kind: kind.clone(),
            new_tiles,
            existing,
        });
    }
}

/// Observer for `BeltDragCompleted`. Handles cost deduction, existing belt
/// direction updates, and spawning new belt/splitter/sorter entities.
pub fn on_belt_drag_completed(
    on: On<BeltDragCompleted>,
    mut commands: Commands,
    mut belt_write: Query<(&TilePosition, &mut BeltSlots)>,
    mut player_query: Query<&mut Inventory, With<Player>>,
    registry: Res<BuildingRegistry>,
    cfg: Res<MapConfig>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    let ev = on.event();
    let Some(def) = registry.get(&ev.kind) else {
        return;
    };
    let tile_size = cfg.tile_size;

    if !ev.new_tiles.is_empty() {
        let mut player_inv = match player_query.single_mut() {
            Ok(inv) => inv,
            Err(_) => return,
        };
        let scaled_cost: Vec<Cost> = def
            .cost
            .iter()
            .map(|c| Cost {
                resource: c.resource.clone(),
                amount: c.amount * ev.new_tiles.len() as u32,
            })
            .collect();
        if !can_afford(&player_inv, &scaled_cost) {
            toast_queue.0.push("Not enough resources".to_string());
            return;
        }
        deduct_cost(&mut player_inv, &scaled_cost);
    }

    if ev.new_tiles.is_empty() && ev.existing.is_empty() {
        return;
    }

    if def.belt.is_some() {
        let num_slots = def.belt.as_ref().map_or(2, |b| b.slots);
        let speed = def.belt.as_ref().map_or(2.0, |b| b.speed);

        for &(bx, by, dir) in &ev.existing {
            if let Some((_, mut bs)) = belt_write
                .iter_mut()
                .find(|(pos, _)| pos.x == bx && pos.y == by)
            {
                if bs.direction != dir {
                    bs.direction = dir;
                    bs.slot_positions = compute_slot_positions(bx, by, dir, num_slots, tile_size);
                }
            }
        }

        for &(bx, by, dir) in &ev.new_tiles {
            let cx = bx as f32 * tile_size;
            let cy = by as f32 * tile_size;
            let slot_positions = compute_slot_positions(bx, by, dir, num_slots, tile_size);
            let items: Vec<Option<crate::economy::belt::ItemOnBelt>> =
                vec![None; num_slots as usize];
            let slot_sprites: Vec<Option<Entity>> = vec![None; num_slots as usize];
            let angle = match dir {
                Direction::East => 0.0,
                Direction::North => std::f32::consts::FRAC_PI_2,
                Direction::West => std::f32::consts::PI,
                Direction::South => -std::f32::consts::FRAC_PI_2,
            };

            let belt_components = (
                Building {
                    kind: def.id.clone(),
                    name: def.name.clone(),
                },
                Inventory::new(),
                OccupiedTiles(vec![(bx, by)]),
                TilePosition { x: bx, y: by },
                BeltSlots {
                    direction: dir,
                    items,
                    slot_sprites,
                    slot_positions,
                    speed,
                },
                Transform::from_xyz(cx, cy, 2.0).with_rotation(Quat::from_rotation_z(angle)),
            );

            if def.id == "splitter" {
                commands.spawn((
                    belt_components,
                    Splitter {
                        counter: 0,
                        outputs: 2,
                        input_direction: None,
                    },
                    Active(true),
                ));
            } else if def.id == "sorter" {
                let filter = def.default_filter.clone().unwrap_or_else(|| "iron_ore".to_string());
                commands.spawn((
                    belt_components,
                    Sorter {
                        filter: ResourceId(filter),
                        inverted: false,
                    },
                    Active(true),
                ));
            } else {
                commands.spawn((belt_components, Active(true)));
            }
        }
    } else if def.drag_placement {
        for &(bx, by, _dir) in &ev.new_tiles {
            let cx = bx as f32 * tile_size;
            let cy = by as f32 * tile_size;
            commands.spawn((
                Building {
                    kind: def.id.clone(),
                    name: def.name.clone(),
                },
                Inventory::new(),
                OccupiedTiles(vec![(bx, by)]),
                TilePosition { x: bx, y: by },
                Transform::from_xyz(cx, cy, 2.0),
                Active(true),
            ));
        }
    }
}

// ── Deconstruct drag ──

pub fn track_deconstruct_drag(
    mut commands: Commands,
    mut drag: ResMut<DeconstructDrag>,
    deconstruct: Res<DeconstructMode>,
    cfg: Res<MapConfig>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
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

    let tile_size = cfg.tile_size;

    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_transform)) = camera.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else {
        return;
    };

    let tx = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let ty = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

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

/// Observer for `DeconstructAreaEvent`. Despawns all buildings in the zone.
pub fn on_deconstruct_area(
    on: On<DeconstructAreaEvent>,
    mut commands: Commands,
    spatial: Res<SpatialRegistry>,
    building_query: Query<(&Building, &OccupiedTiles)>,
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

    for entity in entities {
        let Ok((building, _tiles)) = building_query.get(entity) else {
            continue;
        };

        if let Some(def) = registry.get(&building.kind) {
            if !def.can_deconstruct {
                continue;
            }
            for c in &def.cost {
                let refund = (c.amount as f32 * def.refund_ratio).ceil() as u32;
                if refund > 0 {
                    if let Ok(mut player_inv) = player_query.single_mut() {
                        player_inv.add(&c.resource, refund);
                    }
                    refund_names.push(format!("{} {}", refund, c.resource.display_name()));
                }
            }
        }

        if let Ok(belt_slots) = belt_slots_query.get(entity) {
            for sprite_entity in belt_slots.slot_sprites.iter().flatten() {
                commands.entity(*sprite_entity).despawn();
            }
        }

        commands.entity(entity).despawn();
        count += 1;
    }

    if count > 0 {
        toast_queue.0.push(format!(
            "Zone deconstruct: {} building(s) removed (+{})",
            count,
            refund_names.join(", ")
        ));
    }
}

// ── Build click (blueprint placement) ──

#[allow(clippy::too_many_arguments)]
pub fn handle_build_click(
    mut commands: Commands,
    build_mode: Res<BuildMode>,
    cfg: Res<MapConfig>,
    spatial: Res<SpatialRegistry>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    deposits: Query<(Entity, &TilePosition, &ResourceDeposit)>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
    registry: Res<BuildingRegistry>,
    mut toast_queue: ResMut<ToastQueue>,
    mut chunk_grid: ResMut<ChunkGrid>,
    ui_blocking: Res<UiIsBlocking>,
    crops: Query<(Entity, &Crop, &Transform)>,
) {
    if ui_blocking.0 {
        return;
    }
    let tile_size = cfg.tile_size;

    let Some(ref kind) = build_mode.0 else { return };
    if !bindings.just_pressed("place", &keys, &buttons) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_transform)) = camera.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else {
        return;
    };

    let tx = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let ty = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

    let def = match registry.get(kind) {
        Some(d) => d,
        None => return,
    };

    let (tw, th) = def.tile_size;

    // Buildings with belt properties or drag_placement are handled by track_belt_drag
    if def.belt.is_some() || def.drag_placement {
        return;
    }

    let footprint = compute_footprint(tx, ty, tw, th);

    if def.requires_deposit {
        let deposit_data = deposits
            .iter()
            .find(|(_, pos, _)| pos.x == tx && pos.y == ty);
        let Some((deposit_entity, _, res_dep)) = deposit_data else {
            toast_queue.0.push("No resource deposit here".to_string());
            return;
        };
        if !spatial.tiles_are_free(&footprint) {
            toast_queue.0.push("Tile already occupied".to_string());
            return;
        }

        if !cfg.infinite_deposits {
            let cx = tx.div_euclid(CHUNK_SIZE as i32);
            let cy = ty.div_euclid(CHUNK_SIZE as i32);
            let dx = tx.rem_euclid(CHUNK_SIZE as i32) as u32;
            let dy = ty.rem_euclid(CHUNK_SIZE as i32) as u32;
            chunk_grid.set_deposit_amount(cx, cy, dx, dy, 0);
            commands.trigger(DespawnDeposit(deposit_entity));
        }

        let cx = (tx as f32 + (tw as f32 - 1.0) * 0.5) * tile_size;
        let cy = (ty as f32 + (th as f32 - 1.0) * 0.5) * tile_size;
        let deposit_resource = res_dep.resource.clone();
        let mine_recipe = format!("mine_{}", deposit_resource);
        let interval = def
            .production
            .as_ref()
            .map(|p| p.interval_sec)
            .unwrap_or(2.0);
        let mut e = commands.spawn((
            UnbuiltBuilding,
            Miner,
            Building {
                kind: def.id.clone(),
                name: def.name.clone(),
            },
            Inventory::new(),
            OccupiedTiles(footprint),
            Transform::from_xyz(cx, cy, 2.0),
            TilePosition { x: tx, y: ty },
            Assembler {
                production_timer: 0.0,
                interval,
                recipe_id: mine_recipe,
            },
            ProductionCounter::default(),
            DiscoveredRecipes::default(),
        ));
        if def.power_consumption > 0.0 {
            e.insert(PowerConsumer {
                draw: def.power_consumption,
                satisfied: false,
            });
        }
        return;
    }

    if !spatial.tiles_are_free(&footprint) {
        toast_queue.0.push("Tile occupied".to_string());
        return;
    }

    // Despawn any crops on the building footprint
    for (crop_entity, _, crop_tf) in crops.iter() {
        let ctx = (crop_tf.translation.x / tile_size).round() as i32;
        let cty = (crop_tf.translation.y / tile_size).round() as i32;
        if footprint.iter().any(|&(fx, fy)| fx == ctx && fy == cty) {
            commands.entity(crop_entity).try_despawn();
        }
    }

    let cx = (tx as f32 + (tw as f32 - 1.0) * 0.5) * tile_size;
    let cy = (ty as f32 + (th as f32 - 1.0) * 0.5) * tile_size;

    let base = (
        UnbuiltBuilding,
        Building {
            kind: def.id.clone(),
            name: def.name.clone(),
        },
        OccupiedTiles(footprint),
        TilePosition { x: tx, y: ty },
        Transform::from_xyz(cx, cy, 2.0),
    );

    let inv = if def.inventory_capacity > 0 {
        Inventory::with_capacity(def.inventory_capacity)
    } else {
        Inventory::new()
    };

    let do_power_consumer = def.power_consumption > 0.0;
    let do_power_producer = def.power_generation > 0.0;
    let do_power_pole = def.power_pole_range > 0.0;

    if let Some(default_recipe) = &def.default_recipe {
        let interval = def.production_interval.unwrap_or(2.0);
        let mut e = commands.spawn((
            base,
            Assembler {
                production_timer: 0.0,
                interval,
                recipe_id: default_recipe.clone(),
            },
            inv,
            ProductionCounter::default(),
            DiscoveredRecipes::default(),
        ));
        if do_power_consumer {
            e.insert(PowerConsumer {
                draw: def.power_consumption,
                satisfied: false,
            });
        }
        if do_power_producer {
            e.insert(PowerProducer {
                output: def.power_generation,
            });
        }
        if do_power_pole {
            e.insert(PowerPole {
                range: def.power_pole_range,
            });
        }
    } else if def.id == "turret" {
        let stats = def.combat.as_ref().expect("turret def missing combat");
        let mut e = commands.spawn((
            base,
            inv,
            TurretCombat {
                damage: stats.damage,
                range_sq: stats.range,
                fire_interval: stats.fire_rate_sec,
                timer: 0.0,
                projectile_speed: stats.projectile_speed,
            },
        ));
        if do_power_consumer {
            e.insert(PowerConsumer {
                draw: def.power_consumption,
                satisfied: false,
            });
        }
        if do_power_producer {
            e.insert(PowerProducer {
                output: def.power_generation,
            });
        }
        if do_power_pole {
            e.insert(PowerPole {
                range: def.power_pole_range,
            });
        }
    } else if def.id == "storage" {
        let mut e = commands.spawn((base, inv, Storage));
        if do_power_consumer {
            e.insert(PowerConsumer {
                draw: def.power_consumption,
                satisfied: false,
            });
        }
        if do_power_producer {
            e.insert(PowerProducer {
                output: def.power_generation,
            });
        }
        if do_power_pole {
            e.insert(PowerPole {
                range: def.power_pole_range,
            });
        }
    } else if def.id == "farm" {
        let crop_types = def.crop_types.clone();
        let mut e = commands.spawn((
            base,
            inv,
            Farm {
                crop_index: 0,
                crop_types,
            },
            ProductionCounter::default(),
            DiscoveredRecipes::default(),
        ));
        if do_power_consumer {
            e.insert(PowerConsumer {
                draw: def.power_consumption,
                satisfied: false,
            });
        }
        if do_power_producer {
            e.insert(PowerProducer {
                output: def.power_generation,
            });
        }
        if do_power_pole {
            e.insert(PowerPole {
                range: def.power_pole_range,
            });
        }
    } else if def.id == "archive" {
        let mut e = commands.spawn((base, inv, Archive));
        if do_power_consumer {
            e.insert(PowerConsumer {
                draw: def.power_consumption,
                satisfied: false,
            });
        }
        if do_power_producer {
            e.insert(PowerProducer {
                output: def.power_generation,
            });
        }
        if do_power_pole {
            e.insert(PowerPole {
                range: def.power_pole_range,
            });
        }
    } else {
        let mut e = commands.spawn((base, inv));
        if do_power_consumer {
            e.insert(PowerConsumer {
                draw: def.power_consumption,
                satisfied: false,
            });
        }
        if do_power_producer {
            e.insert(PowerProducer {
                output: def.power_generation,
            });
        }
        if do_power_pole {
            e.insert(PowerPole {
                range: def.power_pole_range,
            });
        }
    }
}

// Auto-construction is now handled by the builder state machine in player.rs
