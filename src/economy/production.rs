use std::collections::HashMap;
use bevy::prelude::*;

use crate::economy::belt::BeltSlots;
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
                resource: prod.resource,
            });
        }
    }
}

pub fn assembler_tick(
    time: Res<Time>,
    recipes: Res<RecipeRegistry>,
    mut assembler_query: Query<(&mut Assembler, &TilePosition)>,
    mut belt_query: Query<(Entity, &TilePosition, &mut BeltSlots)>,
    mut commands: Commands,
) {
    let recipe = match recipes.get("ammo_craft") {
        Some(r) => r,
        None => return,
    };

    let belt_map: HashMap<(i32, i32), Entity> =
        belt_query.iter().map(|(e, pos, _)| ((pos.x, pos.y), e)).collect();

    for (mut assembler, tile_pos) in assembler_query.iter_mut() {
        assembler.production_timer += time.delta_secs();
        while assembler.production_timer >= recipe.time_sec {
            let input_dirs: [(i32, i32, Direction); 4] = [
                (1, 0, Direction::West),
                (-1, 0, Direction::East),
                (0, 1, Direction::South),
                (0, -1, Direction::North),
            ];

            let mut consumed = false;
            for (dx, dy, expected_dir) in input_dirs {
                let ax = tile_pos.x + dx;
                let ay = tile_pos.y + dy;
                if let Some(&belt_entity) = belt_map.get(&(ax, ay)) {
                    if let Ok((_, _, mut bs)) = belt_query.get_mut(belt_entity) {
                        if bs.direction == expected_dir {
                            let last = bs.slots.len() - 1;
                            if let Some(item_entity) = bs.slots[last].take() {
                                commands.entity(item_entity).despawn();
                                consumed = true;
                                break;
                            }
                        }
                    }
                }
            }

            if !consumed {
                break;
            }

            commands.trigger(SpawnBeltItemEvent {
                source_tile: *tile_pos,
                resource: recipe.output_resource,
            });

            assembler.production_timer -= recipe.time_sec;
        }
    }
}