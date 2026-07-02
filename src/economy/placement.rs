use bevy::prelude::*;
use bevy::prelude::*;
use crate::economy::belt::{BeltSlots, compute_slot_positions};
use crate::economy::building::{BuildingCost, BuildingRegistry};
use crate::economy::components::{
    Direction, BuildMode, BeltDirection, BuildPreview, BeltDrag,
    Building, Miner, Assembler, OreDeposit, Ghost, HQ, Produces, TurretCombat,
};
use crate::economy::resource::Inventory;
use crate::core::toast::ToastQueue;
use crate::events::DespawnDeposit;
use crate::map::components::{HoveredTile, TilePosition};
use crate::map::config::MapConfig;
use crate::rendering::{direction_arrow, material_from_color, ShapeCache};

pub fn build_mode_input(
    mut build_mode: ResMut<BuildMode>,
    mut belt_dir: ResMut<BeltDirection>,
    keys: Res<ButtonInput<KeyCode>>,
    cfg: Res<MapConfig>,
    mut placed_belts: Query<(&mut BeltSlots, &mut Text2d, &TilePosition)>,
    registry: Res<BuildingRegistry>,
    hovered: Res<HoveredTile>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let build_ids: Vec<&String> = registry.buildings.iter()
        .filter(|b| b.id != "hq")
        .map(|b| &b.id)
        .collect();
    for (i, key) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5].iter().enumerate() {
        if keys.just_pressed(*key) {
            if let Some(id) = build_ids.get(i) {
                build_mode.0 = match &build_mode.0 {
                    Some(current) if current == *id => None,
                    _ => Some((*id).clone()),
                };
            }
        }
    }

    if keys.just_pressed(KeyCode::KeyR) && build_mode.0.as_deref() == Some("belt") {
        if let Some(pos) = hovered.0 {
            let mut rotated = false;
            for (mut belt, mut text, tile_pos) in placed_belts.iter_mut() {
                if tile_pos.x == pos.x && tile_pos.y == pos.y {
                    belt.direction = belt.direction.next();
                    belt.slot_positions = compute_slot_positions(
                        tile_pos.x, tile_pos.y, belt.direction,
                        belt.slots.len() as u32, cfg.tile_size,
                    );
                    text.0 = direction_arrow(belt.direction).to_string();
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

    if buttons.just_pressed(MouseButton::Right) {
        build_mode.0 = None;
    }
}

// ── Auto-direction ──

fn auto_detect_direction(
    tx: u32, ty: u32,
    producers: &Query<&TilePosition, With<Produces>>,
    belts_query: &Query<(&TilePosition, &BeltSlots)>,
    default: Direction,
) -> Direction {
    let offsets = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let dirs = [Direction::East, Direction::North, Direction::West, Direction::South];

    for (&(dx, dy), &dir) in offsets.iter().zip(dirs.iter()) {
        let nx = tx.wrapping_add_signed(dx);
        let ny = ty.wrapping_add_signed(dy);
        if producers.iter().any(|pos| pos.x == nx && pos.y == ny) {
            return dir;
        }
    }

    for (pos, slots) in belts_query.iter() {
        let (odx, ody) = slots.direction.offset();
        if pos.x.wrapping_add_signed(odx) == tx && pos.y.wrapping_add_signed(ody) == ty {
            return slots.direction;
        }
    }

    default
}

fn auto_detect_direction_from_data(
    tx: u32, ty: u32,
    producers: &Query<&TilePosition, With<Produces>>,
    belt_data: &[((u32, u32), Direction)],
    default: Direction,
) -> Direction {
    let offsets = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let dirs = [Direction::East, Direction::North, Direction::West, Direction::South];

    for (&(dx, dy), &dir) in offsets.iter().zip(dirs.iter()) {
        let nx = tx.wrapping_add_signed(dx);
        let ny = ty.wrapping_add_signed(dy);
        if producers.iter().any(|pos| pos.x == nx && pos.y == ny) {
            return dir;
        }
    }

    for &((px, py), dir) in belt_data {
        let (odx, ody) = dir.offset();
        if px.wrapping_add_signed(odx) == tx && py.wrapping_add_signed(ody) == ty {
            return dir;
        }
    }

    default
}

// ── Click-drag helpers ──

fn compute_line(start: (u32, u32), end: (u32, u32)) -> Vec<(u32, u32, Direction)> {
    let dx = end.0 as i32 - start.0 as i32;
    let dy = end.1 as i32 - start.1 as i32;
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
                let x = (start.0 as i32 + sdx * i) as u32;
                result.push((x, start.1, dir_x));
            }
            for i in 0..=ady {
                let y = (start.1 as i32 + sdy * i) as u32;
                result.push((end.0, y, dir_y));
            }
        } else {
            for i in 0..ady {
                let y = (start.1 as i32 + sdy * i) as u32;
                result.push((start.0, y, dir_y));
            }
            for i in 0..=adx {
                let x = (start.0 as i32 + sdx * i) as u32;
                result.push((x, end.1, dir_x));
            }
        }
    } else if adx > 0 {
        let sdx = dx.signum();
        let dir = if sdx > 0 { Direction::East } else { Direction::West };
        for i in 0..=adx {
            let x = (start.0 as i32 + sdx * i) as u32;
            result.push((x, start.1, dir));
        }
    } else {
        let sdy = dy.signum();
        let dir = if sdy > 0 { Direction::North } else { Direction::South };
        for i in 0..=ady {
            let y = (start.1 as i32 + sdy * i) as u32;
            result.push((start.0, y, dir));
        }
    }

    result
}

// ── Preview ──

#[allow(clippy::too_many_arguments)]
pub fn update_build_preview(
    mut commands: Commands,
    mut preview: ResMut<BuildPreview>,
    build_mode: Res<BuildMode>,
    belt_dir: Res<BeltDirection>,
    cfg: Res<MapConfig>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    buildings: Query<&TilePosition, With<Building>>,
    deposits: Query<&TilePosition, With<OreDeposit>>,
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

    let Some(ref kind) = build_mode.0 else { return };
    let Some(def) = registry.get(kind) else { return };
    let Some(TilePosition { x: tx, y: ty }) = hovered.0 else { return };

    // ── Drag line preview (belt only) ──
    if def.id == "belt" {
        if let Some((sx, sy)) = drag.start_coord {
            let line = compute_line((sx, sy), (tx, ty));
            for &(lx, ly, dir) in &line {
                let has_belt = belts_query.iter().any(|(p, _)| p.x == lx && p.y == ly);
                let valid = (has_belt || tile_is_free(lx, ly, &buildings)) && lx < cfg.width && ly < cfg.height;
                let angle = match dir {
                    Direction::East => 0.0,
                    Direction::North => std::f32::consts::FRAC_PI_2,
                    Direction::West => std::f32::consts::PI,
                    Direction::South => -std::f32::consts::FRAC_PI_2,
                };
                let color = if valid {
                    Color::srgba(0.0, 0.8, 0.0, 0.4)
                } else {
                    Color::srgba(0.8, 0.0, 0.0, 0.3)
                };
                let mat_handle = materials.add(color);
                let cx = lx as f32 * cfg.tile_size;
                let cy = ly as f32 * cfg.tile_size;
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
            }
            return;
        }
    }

    // ── Single-tile preview ──
    let valid = if def.requires_deposit {
        deposits.iter().any(|pos| pos.x == tx && pos.y == ty)
            && !buildings.iter().any(|pos| pos.x == tx && pos.y == ty)
    } else {
        tile_is_free(tx, ty, &buildings)
    };

    let color = if valid {
        Color::srgba(0.0, 0.8, 0.0, 0.4)
    } else {
        Color::srgba(0.8, 0.0, 0.0, 0.3)
    };
    let mat_handle = materials.add(color);
    let z = 1.8;
    let cx = tx as f32 * cfg.tile_size;
    let cy = ty as f32 * cfg.tile_size;

    let entity = if def.id == "belt" {
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
        let belt_entity = commands.spawn((
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
            let nx = tx.wrapping_add_signed(dx);
            let ny = ty.wrapping_add_signed(dy);
            let is_input = producers.iter().any(|pos| pos.x == nx && pos.y == ny)
                || belts_query.iter().any(|(pos, slots)| {
                    let (odx, ody) = slots.direction.offset();
                    pos.x.wrapping_add_signed(odx) == tx && pos.y.wrapping_add_signed(ody) == ty
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

        belt_entity
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

fn can_afford(hq_inv: &Inventory, cost: &[BuildingCost]) -> bool {
    cost.iter().all(|c| hq_inv.get(c.resource) >= c.amount)
}

fn deduct_cost(hq_inv: &mut Inventory, cost: &[BuildingCost]) {
    for c in cost {
        hq_inv.remove(c.resource, c.amount);
    }
}

fn tile_is_free(tx: u32, ty: u32, buildings: &Query<&TilePosition, With<Building>>) -> bool {
    !buildings.iter().any(|pos| pos.x == tx && pos.y == ty)
}

// ── Belt click/drag ──

#[allow(clippy::too_many_arguments)]
pub fn handle_belt_placement(
    mut commands: Commands,
    mut drag: ResMut<BeltDrag>,
    build_mode: Res<BuildMode>,
    belt_dir: Res<BeltDirection>,
    cfg: Res<MapConfig>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    buildings: Query<&TilePosition, With<Building>>,
    producers: Query<&TilePosition, With<Produces>>,
    mut belt_params: ParamSet<(
        Query<(&TilePosition, &BeltSlots)>,
        Query<(&TilePosition, &mut BeltSlots, &mut Text2d)>,
    )>,
    mut hq_query: Query<&mut Inventory, With<HQ>>,
    buttons: Res<ButtonInput<MouseButton>>,
    registry: Res<BuildingRegistry>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    let Some(ref kind) = build_mode.0 else {
        drag.start_coord = None;
        return;
    };
    if kind != "belt" {
        drag.start_coord = None;
        return;
    }
    let Some(def) = registry.get("belt") else { return };
    let tile_size = cfg.tile_size;
    let grid_w = cfg.width;
    let grid_h = cfg.height;

    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_transform)) = camera.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else { return };

    let tile_x = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let tile_y = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

    if tile_x < 0 || tile_y < 0 || tile_x >= grid_w as i32 || tile_y >= grid_h as i32 {
        if buttons.just_released(MouseButton::Left) {
            drag.start_coord.take();
        }
        return;
    }

    let tx = tile_x as u32;
    let ty = tile_y as u32;

    if buttons.just_pressed(MouseButton::Left) {
        let belt_data: Vec<((u32, u32), Direction)> = {
            let read = belt_params.p0();
            read.iter().map(|(pos, bs)| ((pos.x, pos.y), bs.direction)).collect()
        };
        let has_belt = belt_data.iter().any(|&((px, py), _)| px == tx && py == ty);
        let is_free = tile_is_free(tx, ty, &buildings);
        if has_belt || is_free {
            drag.start_coord = Some((tx, ty));
        } else {
            toast_queue.0.push("Tile occupied".to_string());
        }
        return;
    }

    if buttons.just_released(MouseButton::Left) {
        let Some(start) = drag.start_coord.take() else { return };

        // Collect belt data once, release p0 borrow before using p1
        let belt_data: Vec<((u32, u32), Direction)> = {
            let read = belt_params.p0();
            read.iter().map(|(pos, bs)| ((pos.x, pos.y), bs.direction)).collect()
        };

        let line = compute_line(start, (tx, ty));
        let single = line.len() == 1;

        // Separate existing belts from new tiles
        let mut existing: Vec<(u32, u32, Direction)> = Vec::new();
        let mut new_tiles: Vec<(u32, u32, Direction)> = Vec::new();

        for &(bx, by, base_dir) in &line {
            let dir = if single {
                auto_detect_direction_from_data(bx, by, &producers, &belt_data, belt_dir.0)
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

        // Deduct cost only for new belts
        if !new_tiles.is_empty() {
            let mut hq_inv = match hq_query.single_mut() {
                Ok(inv) => inv,
                Err(_) => return,
            };

            let scaled_cost: Vec<BuildingCost> = def.cost.iter()
                .map(|c| BuildingCost { resource: c.resource, amount: c.amount * new_tiles.len() as u32 })
                .collect();

            if !can_afford(&hq_inv, &scaled_cost) {
                toast_queue.0.push("Not enough resources".to_string());
                return;
            }

            deduct_cost(&mut hq_inv, &scaled_cost);
        }

        let num_slots = def.belt.as_ref().map_or(4, |b| b.slots);
        let speed = def.belt.as_ref().map_or(2.0, |b| b.speed);

        // Update existing belts (p1 borrow released after each iteration)
        for &(bx, by, dir) in &existing {
            if let Some((_, mut bs, mut text)) = belt_params.p1().iter_mut()
                .find(|(pos, _, _)| pos.x == bx && pos.y == by)
            {
                if bs.direction != dir {
                    bs.direction = dir;
                    bs.slot_positions = compute_slot_positions(bx, by, dir, num_slots, tile_size);
                    text.0 = direction_arrow(dir).to_string();
                }
            }
        }

        // Spawn new belts
        for &(bx, by, dir) in &new_tiles {
            let cx = bx as f32 * tile_size;
            let cy = by as f32 * tile_size;
            let slot_positions = compute_slot_positions(bx, by, dir, num_slots, tile_size);
            let slots = vec![None; num_slots as usize];

            commands.spawn((
                Building { kind: def.id.clone(), name: def.name.clone() },
                Inventory::new(),
                TilePosition { x: bx, y: by },
                BeltSlots { direction: dir, slots, slot_positions, speed },
                Text2d::new(direction_arrow(dir).to_string()),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
                TextLayout::justify(Justify::Center),
                Transform::from_xyz(cx, cy, 2.0),
            ));
        }

        return;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_build_click(
    mut commands: Commands,
    build_mode: Res<BuildMode>,
    cfg: Res<MapConfig>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    deposits: Query<(Entity, &TilePosition), With<OreDeposit>>,
    buildings: Query<&TilePosition, With<Building>>,
    mut hq_query: Query<&mut Inventory, With<HQ>>,
    buttons: Res<ButtonInput<MouseButton>>,
    registry: Res<BuildingRegistry>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    let tile_size = cfg.tile_size;
    let grid_w = cfg.width;
    let grid_h = cfg.height;

    let Some(ref kind) = build_mode.0 else { return };
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_transform)) = camera.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else { return };

    let tile_x = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let tile_y = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

    if tile_x < 0 || tile_y < 0 || tile_x >= grid_w as i32 || tile_y >= grid_h as i32 {
        toast_queue.0.push("Outside map".to_string());
        return;
    }

    let tx = tile_x as u32;
    let ty = tile_y as u32;

    let def = match registry.get(kind) {
        Some(d) => d,
        None => return,
    };

    // Belt is handled by handle_belt_placement
    if def.id == "belt" {
        return;
    }

    if def.requires_deposit {
        let deposit_entity = deposits.iter().find(|(_, pos)| pos.x == tx && pos.y == ty).map(|(e, _)| e);
        let Some(deposit) = deposit_entity else {
            toast_queue.0.push("No ore deposit here".to_string());
            return;
        };
        let already_mined = buildings.iter().any(|pos| pos.x == tx && pos.y == ty);
        if already_mined {
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
        commands.trigger(DespawnDeposit(deposit));

        let mesh = shapes.get_visual(&def.visual);
        commands.spawn((
            Miner { production_timer: 0.0, interval: def.production.as_ref().map(|p| p.interval_sec).unwrap_or(2.0) },
            Building { kind: def.id.clone(), name: def.name.clone() },
            Inventory::new(),
            Mesh2d(mesh),
            MeshMaterial2d(material_from_color(&mut materials, def.color)),
            Transform::from_xyz(tx as f32 * tile_size, ty as f32 * tile_size, 2.0),
            TilePosition { x: tx, y: ty },
            Produces { resource: def.production.as_ref().map(|p| p.resource).unwrap_or(crate::economy::resource::ResourceId::Ore), interval: def.production.as_ref().map(|p| p.interval_sec).unwrap_or(2.0), timer: 0.0 },
        ));
        return;
    }

    if !tile_is_free(tx, ty, &buildings) {
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

    let base = (
        Building { kind: def.id.clone(), name: def.name.clone() },
        Inventory::new(),
        TilePosition { x: tx, y: ty },
    );

    if def.id == "assembler" {
        let mesh = shapes.get_visual(&def.visual);
        commands.spawn((
            base,
            Assembler { production_timer: 0.0, interval: 2.0 },
            Mesh2d(mesh),
            MeshMaterial2d(material_from_color(&mut materials, def.color)),
            Transform::from_xyz(tx as f32 * tile_size, ty as f32 * tile_size, 2.0)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
        ));
    } else if def.id == "turret" {
        let mesh = shapes.get_visual(&def.visual);
        let stats = def.combat.as_ref().expect("turret def missing combat");
        commands.spawn((
            base,
            TurretCombat {
                damage: stats.damage,
                range_sq: stats.range,
                fire_interval: stats.fire_rate_sec,
                timer: 0.0,
            },
            Mesh2d(mesh),
            MeshMaterial2d(material_from_color(&mut materials, def.color)),
            Transform::from_xyz(tx as f32 * tile_size, ty as f32 * tile_size, 2.0),
        ));
    } else {
        let mesh = shapes.get_visual(&def.visual);
        commands.spawn((
            base,
            Mesh2d(mesh),
            MeshMaterial2d(material_from_color(&mut materials, def.color)),
            Transform::from_xyz(tx as f32 * tile_size, ty as f32 * tile_size, 2.0),
        ));
    }
}
