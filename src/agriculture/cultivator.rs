use bevy::prelude::*;
use std::collections::HashSet;

use crate::core::utils::{move_toward, parse_hex_color};
use crate::economy::spatial::SpatialRegistry;
use crate::economy::unit_config::UnitConfig;
use crate::map::config::MapConfig;

use super::components::{
    Crop, CropRegistry, Cultivator, CultivatorState, Farm, PendingDeliveries, PendingDelivery,
};
use super::dim_color;

#[allow(clippy::too_many_arguments)]
pub fn cultivator_ai(
    time: Res<Time>,
    cfg: Res<MapConfig>,
    spatial: Res<SpatialRegistry>,
    crop_registry: Res<CropRegistry>,
    unit_cfg: Res<UnitConfig>,
    crops: Query<(Entity, &Crop, &Transform)>,
    mut set: ParamSet<(
        Query<(Entity, &Farm, &Transform)>,
        Query<&Cultivator, (Without<Crop>, Without<Farm>)>,
        Query<(Entity, &mut Cultivator, &mut Transform), (Without<Crop>, Without<Farm>)>,
    )>,
    mut pending: ResMut<PendingDeliveries>,
    mut commands: Commands,
) {
    // Pre-collect farm positions and crop types for quick lookup
    let mut farm_positions: Vec<(Entity, Vec3)> = Vec::new();
    let mut first_crop_types: Vec<String> = Vec::new();
    let mut first_crop_index: usize = 0;
    for (e, f, tf) in set.p0().iter() {
        if farm_positions.is_empty() {
            first_crop_types = f.crop_types.clone();
            first_crop_index = f.crop_index;
        }
        farm_positions.push((e, tf.translation));
    }

    // Collect reserved crops (already targeted by another cultivator)
    let mut reserved_crops: HashSet<Entity> = HashSet::new();
    let mut reserved_tiles: HashSet<(i32, i32)> = HashSet::new();
    for c in set.p1().iter() {
        match c.state {
            CultivatorState::MovingToHarvest(target) => {
                reserved_crops.insert(target);
            }
            CultivatorState::MovingToPlant(tx, ty) => {
                reserved_tiles.insert((tx, ty));
            }
            _ => {}
        }
    }

    // Same-frame reservation: prevent two Idle cultivators from claiming the same target
    let mut taken_crops = reserved_crops.clone();
    let mut taken_tiles = reserved_tiles.clone();

    let tile_size = cfg.tile_size;
    let cultivator_def = match unit_cfg.get("cultivator") {
        Some(d) => d,
        None => return,
    };
    let speed = cultivator_def.speed;
    let carry_capacity = cultivator_def.carry_capacity;

    for (_entity, mut cultivator, mut transform) in set.p2().iter_mut() {
        match cultivator.state.clone() {
            CultivatorState::Idle => {
                // Find nearest ready crop not already reserved
                let mut nearest_crop: Option<(Entity, f32)> = None;
                for (crop_entity, crop, crop_transform) in crops.iter() {
                    if crop.timer >= crop.duration && !taken_crops.contains(&crop_entity) {
                        let dist = transform.translation.distance(crop_transform.translation);
                        if nearest_crop.map_or(true, |(_, d)| dist < d) {
                            nearest_crop = Some((crop_entity, dist));
                        }
                    }
                }

                let carrying_seeds =
                    cultivator.carried_resource.is_some() && cultivator.carried_amount > 0;

                if let Some((crop_entity, _)) = nearest_crop {
                    if carrying_seeds {
                        if let Some(farm_entity) =
                            find_nearest_farm_pos(transform.translation, &farm_positions)
                        {
                            cultivator.state = CultivatorState::DeliveringToFarm(farm_entity);
                        }
                    } else {
                        taken_crops.insert(crop_entity);
                        cultivator.state = CultivatorState::MovingToHarvest(crop_entity);
                    }
                } else if carrying_seeds {
                    let (cx, cy) =
                        nearest_farm_tile(transform.translation, &farm_positions, tile_size);
                    if let Some((tx, ty)) = find_plantable_tile_spiral(
                        cx,
                        cy,
                        tile_size,
                        &spatial,
                        &crops,
                        &taken_tiles,
                    ) {
                        taken_tiles.insert((tx, ty));
                        cultivator.state = CultivatorState::MovingToPlant(tx, ty);
                    }
                } else {
                    // No seeds, go get some
                    if let Some(farm_entity) =
                        find_nearest_farm_pos(transform.translation, &farm_positions)
                    {
                        cultivator.state = CultivatorState::MovingToFarmForSeeds(farm_entity);
                    }
                }
            }

            CultivatorState::MovingToFarmForSeeds(target) => {
                let farm_pos = farm_positions
                    .iter()
                    .find(|(e, _)| *e == target)
                    .map(|(_, p)| *p);
                if let Some(farm_pos) = farm_pos {
                    let dx = farm_pos.x - transform.translation.x;
                    let dy = farm_pos.y - transform.translation.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < tile_size * 0.5 {
                        // Pick up unlimited seeds
                        let seed_resource = if first_crop_types.is_empty() {
                            crop_registry
                                .crops
                                .keys()
                                .next()
                                .cloned()
                                .unwrap_or_else(|| "wheat".to_string())
                        } else {
                            let idx =
                                first_crop_index.min(first_crop_types.len().saturating_sub(1));
                            first_crop_types[idx].clone()
                        };
                        cultivator.carried_resource = Some(seed_resource);
                        cultivator.carried_amount = carry_capacity;
                        cultivator.state = CultivatorState::Idle;
                    } else {
                        move_toward(&mut transform.translation, farm_pos, speed, time.delta_secs());
                    }
                } else {
                    cultivator.state = CultivatorState::Idle;
                }
            }

            CultivatorState::MovingToPlant(tx, ty) => {
                let target_x = tx as f32 * tile_size;
                let target_y = ty as f32 * tile_size;
                let dx = target_x - transform.translation.x;
                let dy = target_y - transform.translation.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < tile_size * 0.3 {
                    let resource = cultivator.carried_resource.clone().unwrap_or_else(|| {
                        let idx = first_crop_index.min(first_crop_types.len().saturating_sub(1));
                        first_crop_types[idx].clone()
                    });
                    let crop_color = get_crop_color(&crop_registry, &resource);
                    commands.spawn((
                        Crop {
                            resource: resource.clone(),
                            timer: 0.0,
                            duration: get_growth_time(&crop_registry, &resource),
                            color: crop_color,
                        },
                        Sprite {
                            color: dim_color(crop_color, 0.5),
                            custom_size: Some(Vec2::new(tile_size * 0.6, tile_size * 0.6)),
                            ..default()
                        },
                        Transform::from_xyz(target_x, target_y, 0.4),
                    ));
                    // Consume 1 seed
                    if cultivator.carried_amount > 0 {
                        cultivator.carried_amount -= 1;
                    }
                    if cultivator.carried_amount == 0 {
                        cultivator.carried_resource = None;
                    }
                    cultivator.state = CultivatorState::Idle;
                } else {
                    move_toward(&mut transform.translation, Vec3::new(target_x, target_y, 0.0), speed, time.delta_secs());
                }
            }

            CultivatorState::MovingToHarvest(target) => {
                if let Ok((_, crop, crop_transform)) = crops.get(target) {
                    let dx = crop_transform.translation.x - transform.translation.x;
                    let dy = crop_transform.translation.y - transform.translation.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < tile_size * 0.3 {
                        if crop.timer >= crop.duration {
                            let yield_amount = crop_registry
                                .get(&crop.resource)
                                .map(|d| d.yield_amount)
                                .unwrap_or(1);
                            cultivator.carried_resource = Some(crop.resource.clone());
                            cultivator.carried_amount += yield_amount;
                            commands.entity(target).despawn();

                            if cultivator.carried_amount >= carry_capacity {
                                if let Some(farm_entity) =
                                    find_nearest_farm_pos(transform.translation, &farm_positions)
                                {
                                    cultivator.state =
                                        CultivatorState::DeliveringToFarm(farm_entity);
                                } else {
                                    cultivator.state = CultivatorState::Idle;
                                }
                            } else {
                                cultivator.state = CultivatorState::Idle;
                            }
                        } else {
                            cultivator.state = CultivatorState::Idle;
                        }
                    } else {
                        move_toward(&mut transform.translation, crop_transform.translation, speed, time.delta_secs());
                    }
                } else {
                    cultivator.state = CultivatorState::Idle;
                }
            }

            CultivatorState::DeliveringToFarm(target) => {
                let farm_pos = farm_positions
                    .iter()
                    .find(|(e, _)| *e == target)
                    .map(|(_, p)| *p);
                if let Some(farm_pos) = farm_pos {
                    let dx = farm_pos.x - transform.translation.x;
                    let dy = farm_pos.y - transform.translation.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < tile_size * 0.5 {
                        if let Some(ref res) = cultivator.carried_resource {
                            pending.0.push(PendingDelivery {
                                farm_entity: target,
                                resource: res.clone(),
                                amount: cultivator.carried_amount,
                            });
                        }
                        cultivator.carried_resource = None;
                        cultivator.carried_amount = 0;
                        cultivator.state = CultivatorState::Idle;
                    } else {
                        move_toward(&mut transform.translation, farm_pos, speed, time.delta_secs());
                    }
                } else {
                    cultivator.state = CultivatorState::Idle;
                }
            }
        }
    }
}

fn find_plantable_tile_spiral(
    cx: i32,
    cy: i32,
    tile_size: f32,
    spatial: &SpatialRegistry,
    crops: &Query<(Entity, &Crop, &Transform)>,
    reserved_tiles: &HashSet<(i32, i32)>,
) -> Option<(i32, i32)> {
    let max_radius = 50;
    let occupied_crops: HashSet<(i32, i32)> = crops
        .iter()
        .map(|(_, _, tf)| {
            let tx = (tf.translation.x / tile_size).round() as i32;
            let ty = (tf.translation.y / tile_size).round() as i32;
            (tx, ty)
        })
        .collect();

    let mut x = 0i32;
    let mut y = 0i32;
    let mut dx = 1i32;
    let mut dy = 0i32;
    let mut segment_len = 1;
    let mut steps = 0;
    let mut turns = 0;

    for _ in 0..(max_radius * max_radius * 4) {
        if !(x == 0 && y == 0) {
            let tx = cx + x;
            let ty = cy + y;
            if spatial.is_free(tx, ty)
                && !occupied_crops.contains(&(tx, ty))
                && !reserved_tiles.contains(&(tx, ty))
            {
                return Some((tx, ty));
            }
        }

        x += dx;
        y += dy;
        steps += 1;

        if steps == segment_len {
            steps = 0;
            let new_dx = -dy;
            let new_dy = dx;
            dx = new_dx;
            dy = new_dy;
            turns += 1;
            if turns % 2 == 0 {
                segment_len += 1;
            }
        }
    }
    None
}

fn nearest_farm_tile(pos: Vec3, farms: &[(Entity, Vec3)], tile_size: f32) -> (i32, i32) {
    let mut best: Option<(f32, i32, i32)> = None;
    for (_, fp) in farms {
        let dist = pos.distance(*fp);
        let tx = (fp.x / tile_size).round() as i32;
        let ty = (fp.y / tile_size).round() as i32;
        if best.map_or(true, |(d, _, _)| dist < d) {
            best = Some((dist, tx, ty));
        }
    }
    best.map(|(_, tx, ty)| (tx, ty)).unwrap_or_else(|| {
        (
            (pos.x / tile_size).round() as i32,
            (pos.y / tile_size).round() as i32,
        )
    })
}

fn find_nearest_farm_pos(pos: Vec3, farms: &[(Entity, Vec3)]) -> Option<Entity> {
    let mut nearest: Option<(Entity, f32)> = None;
    for (entity, farm_pos) in farms {
        let dist = pos.distance(*farm_pos);
        if nearest.map_or(true, |(_, d)| dist < d) {
            nearest = Some((*entity, dist));
        }
    }
    nearest.map(|(e, _)| e)
}

fn get_growth_time(crop_registry: &CropRegistry, resource: &str) -> f32 {
    crop_registry
        .get(resource)
        .map(|d| d.growth_time)
        .unwrap_or(15.0)
}

fn get_crop_color(crop_registry: &CropRegistry, resource: &str) -> Color {
    crop_registry
        .get(resource)
        .map(|d| parse_hex_color(&d.color))
        .unwrap_or(Color::srgb(0.2, 0.6, 0.2))
}
