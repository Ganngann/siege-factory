use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::economy::components::{Assembler, Building, Direction, Sorter, Splitter};
use crate::economy::recipe::RecipeRegistry;
use crate::economy::resource::{Inventory, ResourceId};
use crate::economy::spatial::SpatialRegistry;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemOnBelt {
    pub resource_id: ResourceId,
    pub acc: f32,
}

#[derive(Component)]
pub struct BeltSlots {
    pub direction: Direction,
    pub slot_positions: Vec<Vec2>,
    pub items: Vec<Option<ItemOnBelt>>,
    pub slot_sprites: Vec<Option<Entity>>,
    pub speed: f32,
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
    time: Res<Time<Fixed>>,
    spatial: Res<SpatialRegistry>,
    mut belt_query: Query<(Entity, &TilePosition, &mut BeltSlots)>,
    mut inventory_query: Query<(Entity, &mut Inventory), (With<Building>, Without<BeltSlots>)>,
    mut splitter_query: Query<(Entity, &TilePosition, &mut Splitter)>,
    sorter_query: Query<(Entity, &TilePosition, &Sorter)>,
) {
    let dt = time.delta_secs();
    let belt_map: HashMap<(i32, i32), Entity> = belt_query
        .iter()
        .map(|(e, pos, _)| ((pos.x, pos.y), e))
        .collect();
    let splitter_map: HashMap<(i32, i32), Entity> = splitter_query
        .iter()
        .map(|(e, pos, _)| ((pos.x, pos.y), e))
        .collect();
    let sorter_map: HashMap<(i32, i32), Entity> = sorter_query
        .iter()
        .map(|(e, pos, _)| ((pos.x, pos.y), e))
        .collect();

    let belt_data: Vec<(Entity, TilePosition, Direction, f32, usize)> = belt_query
        .iter()
        .map(|(e, pos, bs)| (e, *pos, bs.direction, bs.speed, bs.items.len()))
        .collect();

    // Accumulate time on all items
    for (_, _, mut bs) in belt_query.iter_mut() {
        let slot_duration = 1.0 / (bs.speed * bs.items.len() as f32);
        for item in &mut bs.items {
            if let Some(item) = item {
                item.acc = (item.acc + dt).min(slot_duration);
            }
        }
    }

    // Internal advancement: last → first within each belt
    for (belt_entity, _, _, speed, n_slots) in &belt_data {
        let slot_duration = 1.0 / (speed * *n_slots as f32);
        if let Ok((_, _, mut bs)) = belt_query.get_mut(*belt_entity) {
            for i in (0..bs.items.len() - 1).rev() {
                if let Some(ref item) = bs.items[i] {
                    if item.acc >= slot_duration && bs.items[i + 1].is_none() {
                        let mut moved = bs.items[i].take().unwrap();
                        moved.acc -= slot_duration;
                        bs.items[i + 1] = Some(moved);
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
                let ready = bs.items[last]
                    .as_ref()
                    .map(|item| item.acc >= slot_duration)
                    .unwrap_or(false);
                if !ready {
                    continue;
                }
            } else {
                continue;
            }
        }

        // Splitter output routing
        if splitter_map.contains_key(&(belt_pos.x, belt_pos.y)) {
            let mut input_dir: Option<Direction> = None;
            if let Ok(s) = splitter_query.get(*belt_entity) {
                if s.2.input_direction.is_some() {
                    input_dir = s.2.input_direction;
                }
            }
            if input_dir.is_none() {
                for (adj_dx, adj_dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
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

            let mut output_dirs: Vec<Direction> = Vec::new();
            for test_dir in [
                Direction::East,
                Direction::North,
                Direction::West,
                Direction::South,
            ] {
                if Some(test_dir) == input_dir {
                    continue;
                }
                let (tdx, tdy) = test_dir.offset();
                let tx = belt_pos.x + tdx;
                let ty = belt_pos.y + tdy;
                if belt_map.contains_key(&(tx, ty)) {
                    output_dirs.push(test_dir);
                }
            }

            if !output_dirs.is_empty() {
                let counter = splitter_query
                    .get(*belt_entity)
                    .map(|(_, _, s)| s.counter)
                    .unwrap_or(0);
                let start_idx = counter as usize % output_dirs.len();
                for offset in 0..output_dirs.len() {
                    let idx = (start_idx + offset) % output_dirs.len();
                    let (odx, ody) = output_dirs[idx].offset();
                    let out_x = belt_pos.x + odx;
                    let out_y = belt_pos.y + ody;
                    let Some(&target) = belt_map.get(&(out_x, out_y)) else {
                        continue;
                    };
                    let Ok([(_, _, mut bs), (_, _, mut target_bs)]) =
                        belt_query.get_many_mut([*belt_entity, target])
                    else {
                        continue;
                    };
                    if bs.items[last].is_some() {
                        if target_bs.items[0].is_none() {
                            let mut moved = bs.items[last].take().unwrap();
                            moved.acc -= slot_duration;
                            target_bs.items[0] = Some(moved);
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

        // Sorter output routing
        if let Some(&sorter_entity) = sorter_map.get(&(belt_pos.x, belt_pos.y)) {
            let sorter = match sorter_query.get(sorter_entity) {
                Ok((_, _, s)) => s,
                Err(_) => {
                    continue;
                }
            };
            let out_tile = {
                if let Ok((_, _, bs)) = belt_query.get(*belt_entity) {
                    bs.items[last].as_ref().and_then(|item| {
                        let matches_filter = item.resource_id == sorter.filter;
                        let use_side = if sorter.inverted {
                            !matches_filter
                        } else {
                            matches_filter
                        };
                        let (out_dx, out_dy) = if use_side {
                            let side_dir = dir.next();
                            side_dir.offset()
                        } else {
                            (dx, dy)
                        };
                        let out_x = belt_pos.x + out_dx;
                        let out_y = belt_pos.y + out_dy;
                        if belt_map.contains_key(&(out_x, out_y)) {
                            Some((out_x, out_y))
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            };
            if let Some((out_x, out_y)) = out_tile {
                if let Some(&target) = belt_map.get(&(out_x, out_y)) {
                    if let Ok([(_, _, mut bs), (_, _, mut target_bs)]) =
                        belt_query.get_many_mut([*belt_entity, target])
                    {
                        if target_bs.items[0].is_none() {
                            if bs.items[last].is_some() {
                                let mut moved = bs.items[last].take().unwrap();
                                moved.acc -= slot_duration;
                                target_bs.items[0] = Some(moved);
                            }
                        }
                    }
                }
            }
            continue;
        }

        // Normal cross-belt transfer (belt→belt)
        if let Some(&next_belt) = belt_map.get(&(nx, ny)) {
            if *belt_entity == next_belt {
                continue;
            }
            if let Ok([(_, _, mut bs), (_, _, mut next_bs)]) =
                belt_query.get_many_mut([*belt_entity, next_belt])
            {
                if let Some(ref item) = bs.items[last] {
                    if item.acc >= slot_duration && next_bs.items[0].is_none() {
                        let mut moved = bs.items[last].take().unwrap();
                        moved.acc -= slot_duration;
                        next_bs.items[0] = Some(moved);
                    }
                }
            }
        } else if let Some(inv_entity) = spatial.at(nx, ny) {
            // Building deposit (belt→building inventory)
            if let Ok((_, _, mut bs)) = belt_query.get_mut(*belt_entity) {
                if let Some(ref item) = bs.items[last] {
                    if item.acc >= slot_duration {
                        let taken = bs.items[last].take().unwrap();
                        if let Ok((_, mut inv)) = inventory_query.get_mut(inv_entity) {
                            if !inv.is_full() {
                                inv.add(&taken.resource_id, 1);
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
    _cfg: Res<MapConfig>,
) {
    for (belt_pos, mut bs) in belt_query.iter_mut() {
        if bs.items[0].is_some() {
            continue;
        }
        let (odx, ody) = bs.direction.offset();
        let src_x = belt_pos.x - odx;
        let src_y = belt_pos.y - ody;
        if let Some(inv_entity) = spatial.at(src_x, src_y) {
            if let Ok((_, mut inv)) = inventory_query.get_mut(inv_entity) {
                if let Ok(Some(asm)) = assembler_query.get(inv_entity) {
                    if let Some(recipe) = recipes.get(&asm.recipe_id) {
                        let output_res = recipe
                            .output
                            .iter()
                            .find(|(r, _)| inv.get(r) > 0)
                            .map(|(r, _)| r.clone());
                        if let Some(res) = output_res {
                            if inv.remove(&res, 1) {
                                bs.items[0] = Some(ItemOnBelt {
                                    resource_id: res,
                                    acc: 0.0,
                                });
                            }
                        }
                        continue;
                    }
                }
                let first_key = inv.resources.keys().next().cloned();
                if let Some(res) = first_key {
                    if inv.remove(&res, 1) {
                        bs.items[0] = Some(ItemOnBelt {
                            resource_id: res,
                            acc: 0.0,
                        });
                    }
                }
            }
        }
    }
}
