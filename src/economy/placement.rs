use bevy::prelude::*;
use crate::economy::belt::{BeltItem, BeltSlots, compute_slot_positions};
use crate::economy::building::{BuildingCost, BuildingRegistry};
use crate::economy::components::{
    Direction, BuildMode, BeltDirection, BuildPreview, BeltDrag, DeconstructMode, DeconstructDrag,
    Building, Miner, Assembler, ResourceDeposit, Ghost, HQ, OccupiedTiles,
    Produces, TurretCombat, Storage, Splitter, Sorter,
};
use crate::economy::resource::{ResourceId, Inventory};
use crate::core::input::KeyBindings;
use crate::core::toast::ToastQueue;
use crate::events::{BeltDragCompleted, DeconstructAreaEvent, DespawnDeposit};
use crate::map::components::{HoveredTile, TilePosition};
use crate::map::config::MapConfig;
use crate::map::tile_grid::{ChunkGrid, CHUNK_SIZE};
use crate::rendering::{direction_arrow, ShapeCache, TextureCache, texture_stem};

pub fn build_mode_input(
    mut build_mode: ResMut<BuildMode>,
    mut deconstruct: ResMut<DeconstructMode>,
    mut belt_dir: ResMut<BeltDirection>,
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    cfg: Res<MapConfig>,
    mut placed_belts: Query<(&mut BeltSlots, &TilePosition)>,
    hovered: Res<HoveredTile>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if keys.just_pressed(bindings.key("build_rotate")) && build_mode.0.as_deref() == Some("belt") {
        if let Some(pos) = hovered.0 {
            let mut rotated = false;
            for (mut belt, tile_pos) in placed_belts.iter_mut() {
                if tile_pos.x == pos.x && tile_pos.y == pos.y {
                    belt.direction = belt.direction.next();
                    belt.slot_positions = compute_slot_positions(
                        tile_pos.x, tile_pos.y, belt.direction,
                        belt.slots.len() as u32, cfg.tile_size,
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

fn auto_detect_direction(
    tx: i32, ty: i32,
    producers: &Query<&TilePosition, With<Produces>>,
    belts_query: &Query<(&TilePosition, &BeltSlots)>,
    default: Direction,
) -> Direction {
    let offsets = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let dirs = [Direction::East, Direction::North, Direction::West, Direction::South];

    for (&(dx, dy), &dir) in offsets.iter().zip(dirs.iter()) {
        let nx = tx + dx;
        let ny = ty + dy;
        if producers.iter().any(|pos| pos.x == nx && pos.y == ny) {
            return dir;
        }
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
    tx: i32, ty: i32,
    producers: &Query<&TilePosition, With<Produces>>,
    belt_data: &[((i32, i32), Direction)],
    default: Direction,
) -> Direction {
    let offsets = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let dirs = [Direction::East, Direction::North, Direction::West, Direction::South];

    for (&(dx, dy), &dir) in offsets.iter().zip(dirs.iter()) {
        let nx = tx + dx;
        let ny = ty + dy;
        if producers.iter().any(|pos| pos.x == nx && pos.y == ny) {
            return dir;
        }
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
        let dir_x = if sdx > 0 { Direction::East } else { Direction::West };
        let dir_y = if sdy > 0 { Direction::North } else { Direction::South };

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
        let dir = if sdx > 0 { Direction::East } else { Direction::West };
        for i in 0..=adx {
            result.push((start.0 + sdx * i, start.1, dir));
        }
    } else {
        let sdy = dy.signum();
        let dir = if sdy > 0 { Direction::North } else { Direction::South };
        for i in 0..=ady {
            result.push((start.0, start.1 + sdy * i, dir));
        }
    }

    result
}

// ── Multi-tile helpers ──

fn tile_is_free(tx: i32, ty: i32, occupied: &Query<&OccupiedTiles, With<Building>>) -> bool {
    !occupied.iter().any(|tiles| tiles.0.iter().any(|&(x, y)| x == tx && y == ty))
}

fn tiles_are_free(tiles: &[(i32, i32)], occupied: &Query<&OccupiedTiles, With<Building>>) -> bool {
    tiles.iter().all(|&(tx, ty)| tile_is_free(tx, ty, occupied))
}

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

#[allow(unused_mut, unused_variables)]
pub fn handle_deconstruct_click(
    mut commands: Commands,
    deconstruct: Res<DeconstructMode>,
    cfg: Res<MapConfig>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    occupied: Query<(&OccupiedTiles, &Building, &TilePosition)>,
    mut hq_query: Query<&mut Inventory, With<HQ>>,
    buttons: Res<ButtonInput<MouseButton>>,
    registry: Res<BuildingRegistry>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    if !deconstruct.0 { return; }
    if !buttons.just_pressed(MouseButton::Left) { return; }

    let tile_size = cfg.tile_size;
    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_transform)) = camera.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else { return };

    let tx = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let ty = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

    let building_entry = occupied.iter().find(|(tiles, _, _)| {
        tiles.0.iter().any(|&(x, y)| x == tx && y == ty)
    });

    let Some((_tiles, building, _)) = building_entry else {
        return;
    };

    let def = match registry.get(&building.kind) {
        Some(d) => d,
        None => return,
    };

    if !def.can_deconstruct {
        toast_queue.0.push(format!("Cannot deconstruct {}", building.name));
        return;
    }

    let mut refund_total = 0u32;
    for c in &def.cost {
        let refund = (c.amount as f32 * def.refund_ratio).ceil() as u32;
        if refund > 0 {
            if let Ok(mut hq_inv) = hq_query.single_mut() {
                hq_inv.add(&c.resource, refund);
            }
            refund_total += refund;
        }
    }

    toast_queue.0.push(format!("{} dismantled (refund: {} resources)", building.name, refund_total));
}

pub fn handle_deconstruct_click_v2(
    mut commands: Commands,
    deconstruct: Res<DeconstructMode>,
    cfg: Res<MapConfig>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    building_query: Query<(Entity, &OccupiedTiles, &Building, &TilePosition)>,
    mut hq_query: Query<&mut Inventory, With<HQ>>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
    registry: Res<BuildingRegistry>,
    mut toast_queue: ResMut<ToastQueue>,
    belt_slots_query: Query<&BeltSlots>,
    item_query: Query<Entity, With<BeltItem>>,
) {
    if !deconstruct.0 { return; }
    if !bindings.just_pressed("place", &keys, &buttons) { return; }

    let tile_size = cfg.tile_size;
    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_transform)) = camera.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else { return };

    let tx = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let ty = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

    let Some((entity, _tiles, building, _)) = building_query.iter().find(|(_, tiles, _, _)| {
        tiles.0.iter().any(|&(x, y)| x == tx && y == ty)
    }) else { return };

    let def = match registry.get(&building.kind) {
        Some(d) => d,
        None => return,
    };

    if !def.can_deconstruct {
        toast_queue.0.push(format!("Cannot deconstruct {}", building.name));
        return;
    }

    let mut refund_names = Vec::new();
    for c in &def.cost {
        let refund = (c.amount as f32 * def.refund_ratio).ceil() as u32;
        if refund > 0 {
            if let Ok(mut hq_inv) = hq_query.single_mut() {
                hq_inv.add(&c.resource, refund);
            }
            refund_names.push(format!("{} {}", refund, c.resource.display_name()));
        }
    }

    if let Ok(belt_slots) = belt_slots_query.get(entity) {
        for slot in belt_slots.slots.iter() {
            if let Some(item_entity) = slot {
                if item_query.contains(*item_entity) {
                    commands.entity(*item_entity).despawn();
                }
            }
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
    mut materials: ResMut<Assets<ColorMaterial>>,
    occupied: Query<&OccupiedTiles, With<Building>>,
    deposits: Query<&TilePosition, With<ResourceDeposit>>,
    producers: Query<&TilePosition, With<Produces>>,
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
        let Some(TilePosition { x: tx, y: ty }) = hovered.0 else { return };
        let occupied_here = occupied.iter().any(|tiles| tiles.0.iter().any(|&(x, y)| x == tx && y == ty));
        let color = if occupied_here {
            Color::srgba(0.8, 0.0, 0.0, 0.4)
        } else {
            Color::srgba(0.8, 0.0, 0.0, 0.15)
        };
        commands.spawn((
            Ghost,
            Mesh2d(shapes.rectangle.clone()),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(tx as f32 * cfg.tile_size, ty as f32 * cfg.tile_size, 1.8),
        ));
        return;
    }

    let Some(ref kind) = build_mode.0 else { return };
    let Some(def) = registry.get(kind) else { return };
    let Some(TilePosition { x: tx, y: ty }) = hovered.0 else { return };

    // ── Drag line preview ──
    if def.belt.is_some() || def.drag_placement {
        if let Some((sx, sy)) = drag.start_coord {
            let line = compute_line((sx, sy), (tx, ty));
            for &(lx, ly, dir) in &line {
                let has_belt = belts_query.iter().any(|(p, _)| p.x == lx && p.y == ly);
                let valid = has_belt || tile_is_free(lx, ly, &occupied);
                let color = if valid {
                    Color::srgba(0.0, 0.8, 0.0, 0.4)
                } else {
                    Color::srgba(0.8, 0.0, 0.0, 0.3)
                };
                let mat_handle = materials.add(color);
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
                        Transform::from_xyz(cx, cy, 1.8).with_rotation(Quat::from_rotation_z(angle)),
                        Text2d::new(direction_arrow(dir).to_string()),
                        TextFont::from_font_size(18.0),
                        TextColor(if valid { Color::srgba(0.0, 0.8, 0.0, 0.6) } else { Color::srgba(0.8, 0.0, 0.0, 0.5) }),
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
        deposits.iter().any(|pos| pos.x == tx && pos.y == ty)
            && tiles_are_free(&footprint, &occupied)
    } else {
        tiles_are_free(&footprint, &occupied)
    };

    let color = if valid {
        Color::srgba(0.0, 0.8, 0.0, 0.4)
    } else {
        Color::srgba(0.8, 0.0, 0.0, 0.3)
    };
    let mat_handle = materials.add(color);
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
        let ghost_entity = commands.spawn((
            Ghost,
            Mesh2d(shapes.rectangle.clone()),
            MeshMaterial2d(mat_handle),
            Transform::from_xyz(cx, cy, z).with_rotation(Quat::from_rotation_z(angle)),
            Text2d::new(direction_arrow(dir).to_string()),
            TextFont::from_font_size(18.0),
            TextColor(text_color),
            TextLayout::justify(Justify::Center),
        )).id();

        // Connection indicators
        let offsets = [(1, 0), (0, 1), (-1, 0), (0, -1)];
        let dirs = [Direction::East, Direction::North, Direction::West, Direction::South];
        for (&(dx, dy), &check_dir) in offsets.iter().zip(dirs.iter()) {
            let nx = tx + dx;
            let ny = ty + dy;
            let is_input = producers.iter().any(|pos| pos.x == nx && pos.y == ny)
                || belts_query.iter().any(|(pos, slots)| {
                    let (odx, ody) = slots.direction.offset();
                    pos.x + odx == tx && pos.y + ody == ty
                });
            if is_input || check_dir == dir {
                let indicator_color = if is_input {
                    Color::srgba(0.0, 1.0, 0.0, 0.7)
                } else {
                    Color::srgba(0.3, 0.6, 1.0, 0.7)
                };
                let ix = cx + dx as f32 * cfg.tile_size * 0.4;
                let iy = cy + dy as f32 * cfg.tile_size * 0.4;
                commands.spawn((
                    Ghost,
                    Mesh2d(shapes.circle.clone()),
                    MeshMaterial2d(materials.add(indicator_color)),
                    Transform::from_xyz(ix, iy, z + 0.1).with_scale(Vec3::splat(0.25)),
                ));
            }
        }

        ghost_entity
    } else {
        let mesh = shapes.get_visual(&def.visual);
        commands.spawn((
            Ghost,
            Mesh2d(mesh),
            MeshMaterial2d(mat_handle),
            Transform::from_xyz(cx, cy, z),
        )).id()
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
    mut materials: ResMut<Assets<ColorMaterial>>,
    hovered: Res<HoveredTile>,
    building_query: Query<(&Building, &TilePosition, &OccupiedTiles)>,
    registry: Res<BuildingRegistry>,
) {
    if !deconstruct.0 { return; }
    let Some((sx, sy)) = deconstruct_drag.start_coord else { return };
    let Some(TilePosition { x: tx, y: ty }) = hovered.0 else { return };

    let x1 = sx.min(tx);
    let x2 = sx.max(tx);
    let y1 = sy.min(ty);
    let y2 = sy.max(ty);

    for (building, pos, tiles) in building_query.iter() {
        let in_zone = tiles.0.iter().any(|&(x, y)| x >= x1 && x <= x2 && y >= y1 && y <= y2);
        if !in_zone { continue; }

        let Some(def) = registry.get(&building.kind) else { continue; };
        let (tw, th) = def.tile_size;
        let cx = (pos.x as f32 + (tw as f32 - 1.0) * 0.5) * cfg.tile_size;
        let cy = (pos.y as f32 + (th as f32 - 1.0) * 0.5) * cfg.tile_size;
        let mesh = shapes.get_visual(&def.visual);
        commands.spawn((
            Ghost,
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(Color::srgba(0.8, 0.0, 0.0, 0.45))),
            Transform::from_xyz(cx, cy, 10.0),
        ));
    }

    // Subtle red grid for empty tiles in zone
    for gx in x1..=x2 {
        for gy in y1..=y2 {
            let has_building = building_query.iter().any(|(_, _, tiles)|
                tiles.0.iter().any(|&(x, y)| x == gx && y == gy)
            );
            if !has_building {
                commands.spawn((
                    Ghost,
                    Mesh2d(shapes.rectangle.clone()),
                    MeshMaterial2d(materials.add(Color::srgba(0.8, 0.0, 0.0, 0.12))),
                    Transform::from_xyz(gx as f32 * cfg.tile_size, gy as f32 * cfg.tile_size, 9.9),
                ));
            }
        }
    }
}

fn can_afford(hq_inv: &Inventory, cost: &[BuildingCost]) -> bool {
    cost.iter().all(|c| hq_inv.get(&c.resource) >= c.amount)
}

fn deduct_cost(hq_inv: &mut Inventory, cost: &[BuildingCost]) {
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
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    occupied: Query<&OccupiedTiles, With<Building>>,
    producers: Query<&TilePosition, With<Produces>>,
    belt_read: Query<(&TilePosition, &BeltSlots)>,
    buttons: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
    registry: Res<BuildingRegistry>,
    mut toast_queue: ResMut<ToastQueue>,
) {
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
    let Ok((cam, cam_transform)) = camera.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else { return };

    let tx = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let ty = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

    if buttons.just_pressed(bindings.mouse("place")) {
        let belt_data: Vec<((i32, i32), Direction)> = belt_read
            .iter().map(|(pos, bs)| ((pos.x, pos.y), bs.direction)).collect();
        let has_belt = belt_data.iter().any(|&((px, py), _)| px == tx && py == ty);
        let is_free = tile_is_free(tx, ty, &occupied);
        if has_belt || is_free {
            drag.start_coord = Some((tx, ty));
        } else {
            toast_queue.0.push("Tile occupied".to_string());
        }
        return;
    }

    if buttons.just_released(bindings.mouse("place")) {
        let Some(start) = drag.start_coord.take() else { return };

        let belt_data: Vec<((i32, i32), Direction)> = belt_read
            .iter().map(|(pos, bs)| ((pos.x, pos.y), bs.direction)).collect();

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
    mut hq_query: Query<&mut Inventory, With<HQ>>,
    textures: Res<TextureCache>,
    registry: Res<BuildingRegistry>,
    cfg: Res<MapConfig>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    let ev = on.event();
    let Some(def) = registry.get(&ev.kind) else { return };
    let tile_size = cfg.tile_size;

    if !ev.new_tiles.is_empty() {
        let mut hq_inv = match hq_query.single_mut() {
            Ok(inv) => inv,
            Err(_) => return,
        };
        let scaled_cost: Vec<BuildingCost> = def.cost.iter()
            .map(|c| BuildingCost { resource: c.resource.clone(), amount: c.amount * ev.new_tiles.len() as u32 })
            .collect();
        if !can_afford(&hq_inv, &scaled_cost) {
            toast_queue.0.push("Not enough resources".to_string());
            return;
        }
        deduct_cost(&mut hq_inv, &scaled_cost);
    }

    if ev.new_tiles.is_empty() && ev.existing.is_empty() {
        return;
    }

    if def.belt.is_some() {
        let num_slots = def.belt.as_ref().map_or(2, |b| b.slots);
        let speed = def.belt.as_ref().map_or(2.0, |b| b.speed);

        for &(bx, by, dir) in &ev.existing {
            if let Some((_, mut bs)) = belt_write.iter_mut()
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
            let slots = vec![None; num_slots as usize];
            let angle = match dir {
                Direction::East => 0.0,
                Direction::North => std::f32::consts::FRAC_PI_2,
                Direction::West => std::f32::consts::PI,
                Direction::South => -std::f32::consts::FRAC_PI_2,
            };

            let belt_components = (
                Building { kind: def.id.clone(), name: def.name.clone() },
                Inventory::new(),
                OccupiedTiles(vec![(bx, by)]),
                TilePosition { x: bx, y: by },
                BeltSlots { direction: dir, slots, slot_positions, speed },
                Transform::from_xyz(cx, cy, 2.0).with_rotation(Quat::from_rotation_z(angle)),
                Visibility::default(),
            );

            let stem = texture_stem(&def.id);
            let sprite = Sprite {
                image: textures.base(stem),
                custom_size: Some(Vec2::splat(tile_size)),
                ..default()
            };

            if def.id == "splitter" {
                commands.spawn((
                    belt_components,
                    Splitter { counter: 0, outputs: 2, input_direction: None },
                    sprite,
                ));
            } else if def.id == "sorter" {
                commands.spawn((
                    belt_components,
                    Sorter { filter: ResourceId("iron_ore".to_string()), inverted: false },
                    sprite,
                ));
            } else {
                commands.spawn((
                    belt_components,
                    sprite,
                ));
            }
        }
    } else if def.drag_placement {
        for &(bx, by, _dir) in &ev.new_tiles {
            let cx = bx as f32 * tile_size;
            let cy = by as f32 * tile_size;
            let stem = texture_stem(&def.id);
            commands.spawn((
                Building { kind: def.id.clone(), name: def.name.clone() },
                Inventory::new(),
                OccupiedTiles(vec![(bx, by)]),
                TilePosition { x: bx, y: by },
                Sprite {
                    image: textures.base(stem),
                    custom_size: Some(Vec2::splat(tile_size)),
                    ..default()
                },
                Transform::from_xyz(cx, cy, 2.0),
                Visibility::default(),
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
) {
    if !deconstruct.0 {
        drag.start_coord = None;
        return;
    }

    let tile_size = cfg.tile_size;

    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_transform)) = camera.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else { return };

    let tx = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let ty = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

    if bindings.just_pressed("place", &keys, &buttons) && drag.start_coord.is_none() {
        drag.start_coord = Some((tx, ty));
        return;
    }

    if buttons.just_released(bindings.mouse("place")) {
        let Some(start) = drag.start_coord.take() else { return };
        commands.trigger(DeconstructAreaEvent {
            start: TilePosition { x: start.0, y: start.1 },
            end: TilePosition { x: tx, y: ty },
        });
    }
}

/// Observer for `DeconstructAreaEvent`. Despawns all buildings in the zone.
pub fn on_deconstruct_area(
    on: On<DeconstructAreaEvent>,
    mut commands: Commands,
    building_query: Query<(Entity, &OccupiedTiles, &Building)>,
    belt_slots_query: Query<&BeltSlots>,
    item_query: Query<Entity, With<BeltItem>>,
    mut hq_query: Query<&mut Inventory, With<HQ>>,
    registry: Res<BuildingRegistry>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    let ev = on.event();
    let x1 = ev.start.x.min(ev.end.x);
    let x2 = ev.start.x.max(ev.end.x);
    let y1 = ev.start.y.min(ev.end.y);
    let y2 = ev.start.y.max(ev.end.y);

    let mut count = 0u32;
    let mut refund_names: Vec<String> = Vec::new();

    for (entity, tiles, building) in building_query.iter() {
        let in_zone = tiles.0.iter().any(|&(x, y)| x >= x1 && x <= x2 && y >= y1 && y <= y2);
        if !in_zone { continue; }

        if let Some(def) = registry.get(&building.kind) {
            if def.can_deconstruct {
                for c in &def.cost {
                    let refund = (c.amount as f32 * def.refund_ratio).ceil() as u32;
                    if refund > 0 {
            if let Ok(mut hq_inv) = hq_query.single_mut() {
                hq_inv.add(&c.resource, refund);
            }
                        refund_names.push(format!("{} {}", refund, c.resource.display_name()));
                    }
                }
            }
        }

        if let Ok(belt_slots) = belt_slots_query.get(entity) {
            for slot in belt_slots.slots.iter() {
                if let Some(item_entity) = slot {
                    if item_query.contains(*item_entity) {
                        commands.entity(*item_entity).despawn();
                    }
                }
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

// ── Build click ──

#[allow(clippy::too_many_arguments)]
pub fn handle_build_click(
    mut commands: Commands,
    build_mode: Res<BuildMode>,
    cfg: Res<MapConfig>,
    textures: Res<TextureCache>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    deposits: Query<(Entity, &TilePosition, &ResourceDeposit)>,
    occupied: Query<&OccupiedTiles, With<Building>>,
    mut hq_query: Query<&mut Inventory, With<HQ>>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
    registry: Res<BuildingRegistry>,
    mut toast_queue: ResMut<ToastQueue>,
    mut chunk_grid: ResMut<ChunkGrid>,
) {
    let tile_size = cfg.tile_size;

    let Some(ref kind) = build_mode.0 else { return };
    if !bindings.just_pressed("place", &keys, &buttons) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_transform)) = camera.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else { return };

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

    let attach_children = |commands: &mut Commands, entity: Entity, stem: &str, size: Vec2| {
        if let Some(tex) = textures.owner(stem) {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Sprite { image: tex, custom_size: Some(size), color: Color::srgb(0.2, 0.4, 0.8), ..default() },
                    Transform::default(),
                ));
            });
        }
        if let Some(tex) = textures.level(stem) {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Sprite { image: tex, custom_size: Some(size), color: Color::srgb(0.2, 0.8, 0.2), ..default() },
                    Transform::default(),
                ));
            });
        }
    };

    if def.requires_deposit {
        let deposit_data = deposits.iter().find(|(_, pos, _)| pos.x == tx && pos.y == ty);
        let Some((deposit_entity, _, res_dep)) = deposit_data else {
            toast_queue.0.push("No resource deposit here".to_string());
            return;
        };
        if !tiles_are_free(&footprint, &occupied) {
            toast_queue.0.push("Tile already occupied".to_string());
            return;
        }

        let mut hq_inv = match hq_query.single_mut() {
            Ok(inv) => inv,
            Err(_) => return,
        };

        if !can_afford(&hq_inv, &def.cost) {
            toast_queue.0.push("Not enough ore".to_string());
            return;
        }

        deduct_cost(&mut hq_inv, &def.cost);
        let cx = tx.div_euclid(CHUNK_SIZE as i32);
        let cy = ty.div_euclid(CHUNK_SIZE as i32);
        let dx = tx.rem_euclid(CHUNK_SIZE as i32) as u32;
        let dy = ty.rem_euclid(CHUNK_SIZE as i32) as u32;
        chunk_grid.set_deposit_amount(cx, cy, dx, dy, 0);
        commands.trigger(DespawnDeposit(deposit_entity));

        let cx = (tx as f32 + (tw as f32 - 1.0) * 0.5) * tile_size;
        let cy = (ty as f32 + (th as f32 - 1.0) * 0.5) * tile_size;
        let stem = texture_stem(&def.id);
        let size = Vec2::new(tw as f32 * tile_size, th as f32 * tile_size);
        let deposit_resource = ResourceId(res_dep.resource.clone());
        let entity = commands.spawn((
            Miner { production_timer: 0.0, interval: def.production.as_ref().map(|p| p.interval_sec).unwrap_or(2.0) },
            Building { kind: def.id.clone(), name: def.name.clone() },
            Inventory::new(),
            OccupiedTiles(footprint),
            Sprite { image: textures.base(stem), custom_size: Some(size), ..default() },
            Transform::from_xyz(cx, cy, 2.0),
            Visibility::default(),
            TilePosition { x: tx, y: ty },
            Produces { resource: deposit_resource, interval: def.production.as_ref().map(|p| p.interval_sec).unwrap_or(2.0), timer: 0.0 },
        )).id();
        attach_children(&mut commands, entity, stem, size);
        return;
    }

    if !tiles_are_free(&footprint, &occupied) {
        toast_queue.0.push("Tile occupied".to_string());
        return;
    }

    let mut hq_inv = match hq_query.single_mut() {
        Ok(inv) => inv,
        Err(_) => return,
    };

    if !can_afford(&hq_inv, &def.cost) {
        toast_queue.0.push("Not enough ore".to_string());
        return;
    }

    deduct_cost(&mut hq_inv, &def.cost);

    let cx = (tx as f32 + (tw as f32 - 1.0) * 0.5) * tile_size;
    let cy = (ty as f32 + (th as f32 - 1.0) * 0.5) * tile_size;
    let stem = texture_stem(&def.id);
    let size = Vec2::new(tw as f32 * tile_size, th as f32 * tile_size);

    let base = (
        Building { kind: def.id.clone(), name: def.name.clone() },
        OccupiedTiles(footprint),
        TilePosition { x: tx, y: ty },
        Transform::from_xyz(cx, cy, 2.0),
        Visibility::default(),
        Sprite { image: textures.base(stem), custom_size: Some(size), ..default() },
    );

    let inv = if def.inventory_capacity > 0 {
        Inventory::with_capacity(def.inventory_capacity)
    } else {
        Inventory::new()
    };

    if def.id == "assembler" || def.id == "furnace" {
        let recipe_id = if def.id == "furnace" { "iron_plate" } else { "ammo_craft" };
        let entity = commands.spawn((
            base,
            Assembler { production_timer: 0.0, interval: 2.0, recipe_id: recipe_id.to_string() },
            inv,
        )).id();
        attach_children(&mut commands, entity, stem, size);
    } else if def.id == "turret" {
        let stats = def.combat.as_ref().expect("turret def missing combat");
        let entity = commands.spawn((
            base,
            inv,
            TurretCombat {
                damage: stats.damage,
                range_sq: stats.range,
                fire_interval: stats.fire_rate_sec,
                timer: 0.0,
            },
        )).id();
        attach_children(&mut commands, entity, stem, size);
    } else if def.id == "storage" {
        let entity = commands.spawn((
            base,
            inv,
            Storage,
        )).id();
        attach_children(&mut commands, entity, stem, size);
    } else {
        let entity = commands.spawn((
            base,
            inv,
        )).id();
        attach_children(&mut commands, entity, stem, size);
    }
}
