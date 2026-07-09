use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::economy::components::{
    Assembler, Building, Direction, Sorter, Splitter, UnbuiltBuilding,
};
use crate::economy::recipe::RecipeRegistry;
use crate::economy::resource::{Inventory, ResourceId};
use crate::economy::spatial::SpatialRegistry;
use crate::core::utils::tile_to_world;
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
    let center = tile_to_world(tx, ty, tile_size);
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

fn build_pos_map<T: QueryData>(
    query: &Query<(Entity, &TilePosition, T)>,
) -> HashMap<(i32, i32), Entity> {
    query
        .iter()
        .map(|(e, pos, _)| ((pos.x, pos.y), e))
        .collect()
}

fn transfer_item(
    src: &mut [Option<ItemOnBelt>],
    dst: &mut [Option<ItemOnBelt>],
    src_idx: usize,
    dst_idx: usize,
    slot_duration: f32,
) {
    if let Some(mut item) = src[src_idx].take() {
        item.acc -= slot_duration;
        dst[dst_idx] = Some(item);
    }
}

pub fn advance_belt_slots(
    time: Res<Time<Fixed>>,
    spatial: Res<SpatialRegistry>,
    mut belt_query: Query<(Entity, &TilePosition, &mut BeltSlots)>,
    // SUGGEST: type BuildingInventoryQuery = Query<(Entity, &mut Inventory), (With<Building>, Without<BeltSlots>, Without<UnbuiltBuilding>)> (clippy::type_complexity)
    mut inventory_query: Query<
        (Entity, &mut Inventory),
        (With<Building>, Without<BeltSlots>, Without<UnbuiltBuilding>),
    >,
    mut splitter_query: Query<(Entity, &TilePosition, &mut Splitter)>,
    sorter_query: Query<(Entity, &TilePosition, &Sorter)>,
) {
    let dt = time.delta_secs();
    let belt_map = build_pos_map(&belt_query);
    let splitter_map = build_pos_map(&splitter_query);
    let sorter_map = build_pos_map(&sorter_query);

    let belt_data: Vec<(Entity, TilePosition, Direction, f32, usize, f32)> = belt_query
        .iter()
        .map(|(e, pos, bs)| {
            let slot_duration = 1.0 / (bs.speed * bs.items.len() as f32);
            (
                e,
                *pos,
                bs.direction,
                bs.speed,
                bs.items.len(),
                slot_duration,
            )
        })
        .collect();

    // Accumulate time on all items
    for (belt_entity, _, _, _, _, slot_duration) in &belt_data {
        if let Ok((_, _, mut bs)) = belt_query.get_mut(*belt_entity) {
            for item in bs.items.iter_mut().flatten() {
                item.acc = (item.acc + dt).min(*slot_duration);
            }
        }
    }

    // Internal advancement: last → first within each belt
    for (belt_entity, _, _, _, _, slot_duration) in &belt_data {
        if let Ok((_, _, mut bs)) = belt_query.get_mut(*belt_entity) {
            for i in (0..bs.items.len() - 1).rev() {
                if let Some(ref item) = bs.items[i]
                    && item.acc >= *slot_duration && bs.items[i + 1].is_none() {
                        let (left, right) = bs.items.split_at_mut(i + 1);
                        if let Some(mut item) = left[i].take() {
                            item.acc -= *slot_duration;
                            right[0] = Some(item);
                        }
                    }
            }
        }
    }

    // Cross-belt transfers (belt→belt + belt→splitter/sorter + belt→building)
    for (belt_entity, belt_pos, dir, _, n_slots, slot_duration) in &belt_data {
        let (dx, dy) = dir.offset();
        let nx = belt_pos.x + dx;
        let ny = belt_pos.y + dy;
        let last = n_slots - 1;

        // Does this belt have an item ready in its last slot?
        {
            if let Ok((_, _, bs)) = belt_query.get(*belt_entity) {
                let ready = bs.items[last]
                    .as_ref()
                    .map(|item| item.acc >= *slot_duration)
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
            if let Ok(s) = splitter_query.get(*belt_entity)
                && s.2.input_direction.is_some() {
                    input_dir = s.2.input_direction;
                }
            if input_dir.is_none() {
                for (adj_dx, adj_dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let ax = belt_pos.x + adj_dx;
                    let ay = belt_pos.y + adj_dy;
                    if let Some(&adj_entity) = belt_map.get(&(ax, ay))
                        && let Ok((_, _, adj_bs)) = belt_query.get(adj_entity) {
                            let (bd_x, bd_y) = adj_bs.direction.offset();
                            if bd_x == -adj_dx && bd_y == -adj_dy {
                                input_dir = Some(Direction::from_offset(adj_dx, adj_dy));
                                break;
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
                    if bs.items[last].is_some()
                        && target_bs.items[0].is_none() {
                            transfer_item(
                                &mut bs.items,
                                &mut target_bs.items,
                                last,
                                0,
                                *slot_duration,
                            );
                            if let Ok((_, _, mut s)) = splitter_query.get_mut(*belt_entity) {
                                s.counter = s.counter.wrapping_add(1);
                                s.input_direction = input_dir;
                            }
                            break;
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
            if let Some((out_x, out_y)) = out_tile
                && let Some(&target) = belt_map.get(&(out_x, out_y))
                    && let Ok([(_, _, mut bs), (_, _, mut target_bs)]) =
                        belt_query.get_many_mut([*belt_entity, target])
                        && target_bs.items[0].is_none()
                            && bs.items[last].is_some() {
                                transfer_item(
                                    &mut bs.items,
                                    &mut target_bs.items,
                                    last,
                                    0,
                                    *slot_duration,
                                );
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
                && let Some(ref item) = bs.items[last]
                    && item.acc >= *slot_duration && next_bs.items[0].is_none() {
                        transfer_item(&mut bs.items, &mut next_bs.items, last, 0, *slot_duration);
                    }
        } else if let Some(inv_entity) = spatial.at(nx, ny) {
            // Building deposit (belt→building inventory)
            if let Ok((_, _, mut bs)) = belt_query.get_mut(*belt_entity)
                && let Some(ref item) = bs.items[last]
                    && item.acc >= *slot_duration {
                        let resource = item.resource_id.clone();
                        if let Ok((_, mut inv)) = inventory_query.get_mut(inv_entity)
                            && !inv.is_full() {
                                inv.add(&resource, 1);
                                bs.items[last] = None;
                            }
                    }
        }
    }
}

pub fn building_output_tick(
    spatial: Res<SpatialRegistry>,
    mut belt_query: Query<(&TilePosition, &mut BeltSlots)>,
    // SUGGEST: type BuildingInventoryQuery = Query<(Entity, &mut Inventory), (With<Building>, Without<BeltSlots>, Without<UnbuiltBuilding>)> (clippy::type_complexity)
    mut inventory_query: Query<
        (Entity, &mut Inventory),
        (With<Building>, Without<BeltSlots>, Without<UnbuiltBuilding>),
    >,
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
        if let Some(inv_entity) = spatial.at(src_x, src_y)
            && let Ok((_, mut inv)) = inventory_query.get_mut(inv_entity) {
                if let Ok(Some(asm)) = assembler_query.get(inv_entity)
                    && let Some(recipe) = recipes.get(&asm.recipe_id) {
                        let output_res = recipe
                            .output
                            .iter()
                            .find(|(r, _)| inv.get(r) > 0)
                            .map(|(r, _)| r.clone());
                        if let Some(res) = output_res
                            && inv.remove(&res, 1) {
                                bs.items[0] = Some(ItemOnBelt {
                                    resource_id: res,
                                    acc: 0.0,
                                });
                            }
                        continue;
                    }
                let first_key = inv.first_resource();
                if let Some(res) = first_key
                    && inv.remove(&res, 1) {
                        bs.items[0] = Some(ItemOnBelt {
                            resource_id: res,
                            acc: 0.0,
                        });
                    }
            }
    }
}
