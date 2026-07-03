use std::collections::HashMap;
use bevy::prelude::*;

use crate::economy::belt::{BeltItem, BeltSlots};
use crate::economy::components::{Assembler, Direction, Produces};
use crate::economy::recipe::RecipeRegistry;
use crate::events::SpawnBeltItemEvent;
use crate::map::components::TilePosition;

pub fn production_tick(
    time: Res<Time>,
    mut producers: Query<(&mut Produces, &TilePosition)>,
    mut commands: Commands,
) {
    for (mut prod, tile_pos) in producers.iter_mut() {
        prod.timer += time.delta_secs();
        while prod.timer >= prod.interval {
            prod.timer -= prod.interval;
            commands.trigger(SpawnBeltItemEvent {
                source_tile: *tile_pos,
                resource: prod.resource.clone(),
            });
        }
    }
}

const INPUT_DIRS: [(i32, i32, Direction); 4] = [
    (1, 0, Direction::West),
    (-1, 0, Direction::East),
    (0, 1, Direction::South),
    (0, -1, Direction::North),
];

pub fn assembler_tick(
    time: Res<Time>,
    recipes: Res<RecipeRegistry>,
    mut assembler_query: Query<(&mut Assembler, &TilePosition)>,
    mut belt_query: Query<(Entity, &TilePosition, &mut BeltSlots)>,
    item_query: Query<&BeltItem>,
    mut commands: Commands,
) {
    let belt_map: HashMap<(i32, i32), Entity> =
        belt_query.iter().map(|(e, pos, _)| ((pos.x, pos.y), e)).collect();

    for (mut assembler, tile_pos) in assembler_query.iter_mut() {
        let recipe = match recipes.get(&assembler.recipe_id) {
            Some(r) => r,
            None => continue,
        };

        assembler.production_timer += time.delta_secs();
        while assembler.production_timer >= recipe.time_sec {
            // First pass: verify all inputs are available
            let mut can_produce = true;
            'check: for (req_resource, req_amount) in &recipe.input {
                let mut found = 0u32;
                for (dx, dy, expected_dir) in INPUT_DIRS {
                    if found >= *req_amount { break; }
                    let ax = tile_pos.x + dx;
                    let ay = tile_pos.y + dy;
                    if let Some(&belt_entity) = belt_map.get(&(ax, ay)) {
                        if let Ok((_, _, bs)) = belt_query.get(belt_entity) {
                            if bs.direction == expected_dir {
                                for slot in bs.slots.iter().rev() {
                                    if let Some(item_entity) = slot {
                                        if let Ok(item) = item_query.get(*item_entity) {
                                            if item.resource == *req_resource {
                                                found += 1;
                                                if found >= *req_amount { break; }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if found < *req_amount {
                    can_produce = false;
                    break 'check;
                }
            }

            if !can_produce { break; }

            // Second pass: consume inputs from belts (last slot first)
            for (req_resource, req_amount) in &recipe.input {
                let mut remaining = *req_amount;
                for (dx, dy, expected_dir) in INPUT_DIRS {
                    if remaining == 0 { break; }
                    let ax = tile_pos.x + dx;
                    let ay = tile_pos.y + dy;
                    if let Some(&belt_entity) = belt_map.get(&(ax, ay)) {
                        if let Ok((_, _, mut bs)) = belt_query.get_mut(belt_entity) {
                            if bs.direction == expected_dir {
                                for i in (0..bs.slots.len()).rev() {
                                    if remaining == 0 { break; }
                                    if let Some(item_entity) = bs.slots[i] {
                                        if let Ok(item) = item_query.get(item_entity) {
                                            if item.resource == *req_resource {
                                                commands.entity(item_entity).despawn();
                                                bs.slots[i] = None;
                                                remaining -= 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Produce all outputs
            for (out_resource, out_amount) in &recipe.output {
                for _ in 0..*out_amount {
                    commands.trigger(SpawnBeltItemEvent {
                        source_tile: *tile_pos,
                        resource: out_resource.clone(),
                    });
                }
            }

            assembler.production_timer -= recipe.time_sec;
        }
    }
}
