use std::collections::HashMap;
use bevy::prelude::*;

use crate::economy::resource::{ResourceId, Inventory};
use crate::economy::components::{Assembler, Building, Direction, Splitter, Sorter};
use crate::economy::recipe::RecipeRegistry;
use crate::economy::spatial::SpatialRegistry;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::{TextureCache, item_stem};

#[derive(Component, Clone)]
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
    tx: i32,
    ty: i32,
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



pub fn advance_belt_slots(
    time: Res<Time>,
    mut commands: Commands,
    spatial: Res<SpatialRegistry>,
    mut belt_query: Query<(Entity, &TilePosition, &mut BeltSlots)>,
    mut item_query: Query<&mut BeltItem>,
    mut inventory_query: Query<(Entity, &mut Inventory), (With<Building>, Without<BeltSlots>)>,
    mut splitter_query: Query<(Entity, &TilePosition, &mut Splitter)>,
    sorter_query: Query<(Entity, &TilePosition, &Sorter)>,
) {
    let dt = time.delta_secs();
    let belt_map: HashMap<(i32, i32), Entity> =
        belt_query.iter().map(|(e, pos, _)| ((pos.x, pos.y), e)).collect();
    let splitter_map: HashMap<(i32, i32), Entity> =
        splitter_query.iter().map(|(e, pos, _)| ((pos.x, pos.y), e)).collect();
    let sorter_map: HashMap<(i32, i32), Entity> =
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

    // Cross-belt transfers (belt→belt + belt→splitter/sorter + belt→building)
    for (belt_entity, belt_pos, dir, speed, n_slots) in &belt_data {
        let slot_duration = 1.0 / (speed * *n_slots as f32);
        let (dx, dy) = dir.offset();
        let nx = belt_pos.x + dx;
        let ny = belt_pos.y + dy;
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
                    let ax = belt_pos.x + adj_dx;
                    let ay = belt_pos.y + adj_dy;
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
                let tx = belt_pos.x + tdx;
                let ty = belt_pos.y + tdy;
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
                    let out_x = belt_pos.x + odx;
                    let out_y = belt_pos.y + ody;
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
                            let out_x = belt_pos.x + out_dx;
                            let out_y = belt_pos.y + out_dy;
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
        } else if let Some(inv_entity) = spatial.at(nx, ny) {
            // Building deposit (belt→building inventory)
            if let Ok((_, _, mut bs)) = belt_query.get_mut(*belt_entity) {
                if let Some(item_entity) = bs.slots[last] {
                    if let Ok(item) = item_query.get(item_entity) {
                        if item.acc >= slot_duration {
                            bs.slots[last].take();
                            let resource = item.resource.clone();
                            commands.entity(item_entity).despawn();
                            if let Ok((_, mut inv)) = inventory_query.get_mut(inv_entity) {
                                if !inv.is_full() {
                                    inv.add(&resource, 1);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn building_output_tick(
    spatial: Res<SpatialRegistry>,
    mut belt_query: Query<(&TilePosition, &mut BeltSlots)>,
    mut inventory_query: Query<(Entity, &mut Inventory), (With<Building>, Without<BeltSlots>)>,
    assembler_query: Query<Option<&Assembler>>,
    recipes: Res<RecipeRegistry>,
    mut commands: Commands,
    textures: Res<TextureCache>,
    cfg: Res<MapConfig>,
) {
    let tile_size = cfg.tile_size;

    for (belt_pos, mut bs) in belt_query.iter_mut() {
        if bs.slots[0].is_some() { continue; }
        let (odx, ody) = bs.direction.offset();
        let src_x = belt_pos.x - odx;
        let src_y = belt_pos.y - ody;
        if let Some(inv_entity) = spatial.at(src_x, src_y) {
            if let Ok((_, mut inv)) = inventory_query.get_mut(inv_entity) {
                // For production buildings, only extract recipe outputs
                if let Ok(Some(asm)) = assembler_query.get(inv_entity) {
                    if let Some(recipe) = recipes.get(&asm.recipe_id) {
                        // Find an output resource that exists in inventory
                        let output_res = recipe.output.iter()
                            .find(|(r, _)| inv.get(r) > 0)
                            .map(|(r, _)| r.clone());
                        if let Some(res) = output_res {
                            if inv.remove(&res, 1) {
                                spawn_belt_item(&mut commands, &textures, belt_pos, tile_size, res, &mut bs);
                            }
                        }
                        continue;
                    }
                }
                // Non-production building: extract any resource
                let first_key = inv.resources.keys().next().cloned();
                if let Some(res) = first_key {
                    if inv.remove(&res, 1) {
                        spawn_belt_item(&mut commands, &textures, belt_pos, tile_size, res, &mut bs);
                    }
                }
            }
        }
    }
}

fn spawn_belt_item(
    commands: &mut Commands,
    textures: &TextureCache,
    _belt_pos: &TilePosition,
    _tile_size: f32,
    resource: ResourceId,
    bs: &mut BeltSlots,
) {
    let stem = item_stem(&resource.0);
    let tex = textures.base(stem);
    let spawn_pos = Vec3::new(
        bs.slot_positions[0].x,
        bs.slot_positions[0].y,
        2.5,
    );
    let item_entity = commands.spawn((
        BeltItem { resource, acc: 0.0 },
        Sprite {
            image: tex,
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        Transform::from_translation(spawn_pos),
    )).id();
    bs.slots[0] = Some(item_entity);
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
