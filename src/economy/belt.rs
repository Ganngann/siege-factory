use std::collections::HashMap;
use bevy::prelude::*;

use crate::economy::resource::{ResourceId, Inventory};
use crate::economy::components::{Building, Direction, OccupiedTiles, Splitter, Sorter};
use crate::events::SpawnBeltItemEvent;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::ShapeCache;

#[derive(Component)]
pub struct BeltSlots {
    pub direction: Direction,
    pub slots: Vec<Option<Entity>>,
    pub slot_positions: Vec<Vec2>,
    pub speed: f32,
}

#[derive(Component)]
pub struct BeltItem {
    pub resource: ResourceId,
    pub acc: f32,
}

pub fn compute_slot_positions(
    tx: u32,
    ty: u32,
    direction: Direction,
    num_slots: u32,
    tile_size: f32,
) -> Vec<Vec2> {
    let center = Vec2::new(tx as f32 * tile_size, ty as f32 * tile_size);
    let (dx, dy) = direction.offset();
    let dir_vec = Vec2::new(dx as f32, dy as f32);
    (0..num_slots)
        .map(|i| {
            let fraction = (i as f32 + 0.5) / num_slots as f32;
            let offset = (fraction - 0.5) * tile_size;
            center + dir_vec * offset
        })
        .collect()
}

pub fn belt_item_placer(
    on: On<SpawnBeltItemEvent>,
    mut belt_query: Query<(Entity, &TilePosition, &mut BeltSlots)>,
    mut commands: Commands,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cfg: Res<MapConfig>,
) {
    let tile_size = cfg.tile_size;
    let belt_map: HashMap<(u32, u32), Entity> =
        belt_query.iter().map(|(e, pos, _)| ((pos.x, pos.y), e)).collect();
    let ev = on.event();

    let mut placed = false;
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let ax = ev.source_tile.x.wrapping_add_signed(dx);
        let ay = ev.source_tile.y.wrapping_add_signed(dy);
        if let Some(&belt_entity) = belt_map.get(&(ax, ay)) {
            if let Ok((_, _, mut bs)) = belt_query.get_mut(belt_entity) {
                // Skip belts that point toward the source tile (input belts)
                let (odx, ody) = bs.direction.offset();
                if ax.wrapping_add_signed(odx) == ev.source_tile.x
                    && ay.wrapping_add_signed(ody) == ev.source_tile.y
                {
                    continue;
                }
                if let Some(free_idx) = bs.slots.iter().position(|s| s.is_none()) {
                    let spawn_pos = Vec3::new(
                        ev.source_tile.x as f32 * tile_size,
                        ev.source_tile.y as f32 * tile_size,
                        2.5,
                    );
                    let color = match ev.resource {
                        ResourceId::Ore => Color::srgb(0.7, 0.5, 0.1),
                        ResourceId::Ammo => Color::srgb(0.8, 0.2, 0.2),
                        ResourceId::Energy => Color::srgb(0.2, 0.6, 0.8),
                    };
                    let item_entity = commands.spawn((
                        BeltItem { resource: ev.resource, acc: 0.0 },
                        Mesh2d(shapes.circle.clone()),
                        MeshMaterial2d(materials.add(color)),
                        Transform::from_translation(spawn_pos).with_scale(Vec3::splat(0.25)),
                    )).id();
                    bs.slots[free_idx] = Some(item_entity);
                    placed = true;
                    break;
                }
            }
        }
    }
    if !placed {
        // No free belt slot — item is backed up
    }
}

pub fn advance_belt_slots(
    time: Res<Time>,
    mut commands: Commands,
    mut belt_query: Query<(Entity, &TilePosition, &mut BeltSlots)>,
    mut item_query: Query<&mut BeltItem>,
    mut inventory_query: Query<(Entity, &OccupiedTiles, &mut Inventory), (With<Building>, Without<BeltSlots>)>,
    mut splitter_query: Query<(Entity, &TilePosition, &mut Splitter)>,
    sorter_query: Query<(Entity, &TilePosition, &Sorter)>,
) {
    let dt = time.delta_secs();
    let belt_map: HashMap<(u32, u32), Entity> =
        belt_query.iter().map(|(e, pos, _)| ((pos.x, pos.y), e)).collect();
    let splitter_map: HashMap<(u32, u32), Entity> =
        splitter_query.iter().map(|(e, pos, _)| ((pos.x, pos.y), e)).collect();
    let sorter_map: HashMap<(u32, u32), Entity> =
        sorter_query.iter().map(|(e, pos, _)| ((pos.x, pos.y), e)).collect();

    let belt_data: Vec<(Entity, TilePosition, Direction, f32, usize)> = belt_query
        .iter()
        .map(|(e, pos, bs)| (e, *pos, bs.direction, bs.speed, bs.slots.len()))
        .collect();

    // Accumulate time on all items
    for (_, _, bs) in belt_query.iter() {
        let slot_duration = 1.0 / (bs.speed * bs.slots.len() as f32);
        for slot in &bs.slots {
            if let Some(item_entity) = slot {
                if let Ok(mut item) = item_query.get_mut(*item_entity) {
                    item.acc = (item.acc + dt).min(slot_duration);
                }
            }
        }
    }

    // Internal advancement: last → first within each belt
    for (belt_entity, _, _, speed, n_slots) in &belt_data {
        let slot_duration = 1.0 / (speed * *n_slots as f32);
        if let Ok((_, _, mut bs)) = belt_query.get_mut(*belt_entity) {
            for i in (0..bs.slots.len() - 1).rev() {
                if let Some(item_entity) = bs.slots[i] {
                    if bs.slots[i + 1].is_none() {
                        if let Ok(mut item) = item_query.get_mut(item_entity) {
                            if item.acc >= slot_duration {
                                bs.slots[i + 1] = bs.slots[i].take();
                                item.acc -= slot_duration;
                            }
                        }
                    }
                }
            }
        }
    }

    // Build inventory map from OccupiedTiles (buildings without BeltSlots)
    let inv_map: HashMap<(u32, u32), Entity> =
        inventory_query.iter()
            .flat_map(|(e, tiles, _)| tiles.0.iter().map(move |&(x, y)| ((x, y), e)))
            .collect();

    // Cross-belt transfers (belt→belt + belt→splitter/sorter + belt→building)
    for (belt_entity, belt_pos, dir, speed, n_slots) in &belt_data {
        let slot_duration = 1.0 / (speed * *n_slots as f32);
        let (dx, dy) = dir.offset();
        let nx = belt_pos.x.wrapping_add_signed(dx);
        let ny = belt_pos.y.wrapping_add_signed(dy);
        let last = n_slots - 1;

        // Does this belt have an item ready in its last slot?
        {
            if let Ok((_, _, bs)) = belt_query.get(*belt_entity) {
                if let Some(item_entity) = bs.slots[last] {
                    if let Ok(item) = item_query.get(item_entity) {
                        if !(item.acc >= slot_duration) { continue; }
                    } else { continue; }
                } else { continue; }
            } else { continue; }
        }

        // Splitter output routing: outputs to ALL adjacent belts except input direction
        if splitter_map.contains_key(&(belt_pos.x, belt_pos.y)) {
            // Determine input direction — prefer stored value, fallback to dynamic detection
            let mut input_dir: Option<Direction> = None;
            if let Ok(s) = splitter_query.get(*belt_entity) {
                if s.2.input_direction.is_some() {
                    input_dir = s.2.input_direction;
                }
            }
            if input_dir.is_none() {
                // Scan neighbors for a belt pointing towards us
                for (adj_dx, adj_dy) in [(1,0), (-1,0), (0,1), (0,-1)] {
                    let ax = belt_pos.x.wrapping_add_signed(adj_dx);
                    let ay = belt_pos.y.wrapping_add_signed(adj_dy);
                    if let Some(&adj_entity) = belt_map.get(&(ax, ay)) {
                        if let Ok((_, _, adj_bs)) = belt_query.get(adj_entity) {
                            let (bd_x, bd_y) = adj_bs.direction.offset();
                            if bd_x == -adj_dx && bd_y == -adj_dy {
                                input_dir = Some(Direction::from_offset(adj_dx, adj_dy));
                                break;
                            }
                        }
                    }
                }
            }

            // Collect all output directions (adjacent belts except input direction)
            let mut output_dirs: Vec<Direction> = Vec::new();
            for test_dir in [Direction::East, Direction::North, Direction::West, Direction::South] {
                if Some(test_dir) == input_dir { continue; }
                let (tdx, tdy) = test_dir.offset();
                let tx = belt_pos.x.wrapping_add_signed(tdx);
                let ty = belt_pos.y.wrapping_add_signed(tdy);
                if belt_map.contains_key(&(tx, ty)) {
                    output_dirs.push(test_dir);
                }
            }

            // Round-robin transfer: try each output in order; if one is blocked, try the next
            if !output_dirs.is_empty() {
                let counter = splitter_query.get(*belt_entity)
                    .map(|(_,_,s)| s.counter).unwrap_or(0);
                let start_idx = counter as usize % output_dirs.len();
                for offset in 0..output_dirs.len() {
                    let idx = (start_idx + offset) % output_dirs.len();
                    let (odx, ody) = output_dirs[idx].offset();
                    let out_x = belt_pos.x.wrapping_add_signed(odx);
                    let out_y = belt_pos.y.wrapping_add_signed(ody);
                    let Some(&target) = belt_map.get(&(out_x, out_y)) else { continue; };
                    let Ok([(_, _, mut bs), (_, _, mut target_bs)]) =
                        belt_query.get_many_mut([*belt_entity, target]) else { continue; };
                    if let Some(entity) = bs.slots[last] {
                        if target_bs.slots[0].is_none() {
                            target_bs.slots[0] = bs.slots[last].take();
                            if let Ok(mut item) = item_query.get_mut(entity) {
                                item.acc -= slot_duration;
                            }
                            if let Ok((_, _, mut s)) = splitter_query.get_mut(*belt_entity) {
                                s.counter = s.counter.wrapping_add(1);
                                s.input_direction = input_dir;
                            }
                            break;
                        }
                    }
                }
            }
            continue;
        }

        // Sorter output routing: filter-dependent (straight vs T-foot)
        if let Some(&sorter_entity) = sorter_map.get(&(belt_pos.x, belt_pos.y)) {
            let sorter = match sorter_query.get(sorter_entity) {
                Ok((_, _, s)) => s,
                Err(_) => { continue; }
            };
            // Phase 1: determine output direction
            let out_tile = {
                if let Ok((_, _, bs)) = belt_query.get(*belt_entity) {
                    if let Some(entity) = bs.slots[last] {
                        if let Ok(item) = item_query.get(entity) {
                            let matches_filter = item.resource == sorter.filter;
                            let use_side = if sorter.inverted { !matches_filter } else { matches_filter };
                            let (out_dx, out_dy) = if use_side {
                                let side_dir = dir.next();
                                side_dir.offset()
                            } else {
                                (dx, dy)
                            };
                            let out_x = belt_pos.x.wrapping_add_signed(out_dx);
                            let out_y = belt_pos.y.wrapping_add_signed(out_dy);
                            if belt_map.contains_key(&(out_x, out_y)) { Some((out_x, out_y)) } else { None }
                        } else { None }
                    } else { None }
                } else { None }
            };
            // Phase 2: transfer
            if let Some((out_x, out_y)) = out_tile {
                if let Some(&target) = belt_map.get(&(out_x, out_y)) {
                    if let Ok([(_, _, mut bs), (_, _, mut target_bs)]) =
                        belt_query.get_many_mut([*belt_entity, target])
                    {
                        if target_bs.slots[0].is_none() {
                            if bs.slots[last].is_some() {
                                target_bs.slots[0] = bs.slots[last].take();
                                if let Some(entity) = target_bs.slots[0] {
                                    if let Ok(mut item) = item_query.get_mut(entity) {
                                        item.acc -= slot_duration;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            continue;
        }

        // Normal cross-belt transfer (belt→belt)
        if let Some(&next_belt) = belt_map.get(&(nx, ny)) {
            if *belt_entity == next_belt { continue; }
            if let Ok([(_, _, mut bs), (_, _, mut next_bs)]) =
                belt_query.get_many_mut([*belt_entity, next_belt])
            {
                if let Some(item_entity) = bs.slots[last] {
                    if next_bs.slots[0].is_none() {
                        if let Ok(mut item) = item_query.get_mut(item_entity) {
                            if item.acc >= slot_duration {
                                next_bs.slots[0] = bs.slots[last].take();
                                item.acc -= slot_duration;
                            }
                        }
                    }
                }
            }
        } else if let Some(&inv_entity) = inv_map.get(&(nx, ny)) {
            // Building deposit (belt→building inventory)
            if let Ok((_, _, mut bs)) = belt_query.get_mut(*belt_entity) {
                if let Some(item_entity) = bs.slots[last] {
                    if let Ok(item) = item_query.get(item_entity) {
                        if item.acc >= slot_duration {
                            bs.slots[last].take();
                            let resource = item.resource;
                            commands.entity(item_entity).despawn();
                            if let Ok((_, _, mut inv)) = inventory_query.get_mut(inv_entity) {
                                if !inv.is_full() {
                                    inv.add(resource, 1);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn animate_belt_positions(
    time: Res<Time>,
    cfg: Res<MapConfig>,
    belt_query: Query<&BeltSlots>,
    mut item_query: Query<&mut Transform, With<BeltItem>>,
) {
    let dt = time.delta_secs();
    let tile_size = cfg.tile_size;

    for bs in belt_query.iter() {
        for (slot_idx, occupant) in bs.slots.iter().enumerate() {
            if let Some(item_entity) = occupant {
                if let Ok(mut transform) = item_query.get_mut(*item_entity) {
                    let target = bs.slot_positions[slot_idx];
                    let current = Vec2::new(transform.translation.x, transform.translation.y);
                    let diff = target - current;
                    let step = bs.speed * tile_size * dt;
                    if diff.length() <= step {
                        transform.translation = Vec3::new(target.x, target.y, 2.5);
                    } else {
                        let new_pos = current + diff.normalize() * step;
                        transform.translation = Vec3::new(new_pos.x, new_pos.y, 2.5);
                    }
                }
            }
        }
    }
}
