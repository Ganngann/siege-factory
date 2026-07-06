use bevy::prelude::*;

use crate::core::utils::{move_toward, tile_to_world, world_to_tile};
use crate::economy::building::BuildingRegistry;
use crate::economy::resource::Cost;
use crate::economy::components::{
    Active, Builder, BuilderState, Building, OccupiedTiles, Player, ResourceDeposit,
    UnbuiltBuilding,
};
use crate::economy::resource::{Inventory, ResourceId};
use crate::enemy::components::Health;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::TextureCache;

pub fn setup_player(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    cfg: Res<MapConfig>,
    textures: Res<TextureCache>,
) {
    if !player_query.is_empty() {
        return;
    }

    let (bx, by) = cfg.player_start_position;
    let start_pos = tile_to_world(bx, by, cfg.tile_size);
    let (cx, cy) = (start_pos.x, start_pos.y);

    let inv = crate::economy::resource::Inventory::new();

    commands.spawn((
        Player,
        inv,
        Health {
            current: cfg.player_hp,
            max: cfg.player_hp,
        },
        Transform::from_xyz(cx, cy, 5.0),
        Visibility::default(),
        TilePosition { x: bx, y: by },
        Sprite {
            image: textures.base("player"),
            custom_size: Some(Vec2::new(28.0, 28.0)),
            ..default()
        },
    ));

    commands.spawn((
        Builder {
            state: BuilderState::Idle,
        },
        Transform::from_xyz(cx - 20.0, cy - 20.0, 4.0),
        Visibility::default(),
        Sprite {
            image: textures.base("builder"),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
    ));
}

pub fn player_movement(
    mut player_query: Query<(&mut Transform, &mut TilePosition), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    cfg: Res<MapConfig>,
    mut player_pos: ResMut<PlayerWorldPos>,
) {
    let Ok((mut tf, mut tile_pos)) = player_query.single_mut() else {
        return;
    };

    let speed = cfg.player_speed;
    let mut dx = 0.0;
    let mut dy = 0.0;

    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dy += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dy -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dx -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dx += 1.0;
    }

    if dx != 0.0 || dy != 0.0 {
        let norm = f32::sqrt(dx * dx + dy * dy);
        dx /= norm;
        dy /= norm;
        tf.translation.x += dx * speed * time.delta_secs();
        tf.translation.y += dy * speed * time.delta_secs();

        let (tx, ty) = world_to_tile(tf.translation.truncate(), cfg.tile_size);
        tile_pos.x = tx;
        tile_pos.y = ty;
    }
    player_pos.0 = tf.translation;
}

pub fn builder_work(
    time: Res<Time>,
    cfg: Res<MapConfig>,
    registry: Res<BuildingRegistry>,
    mut builder_query: Query<(&mut Builder, &mut Transform)>,
    mut player_query: Query<(&Transform, &mut Inventory), (With<Player>, Without<Builder>)>,
    mut building_query: Query<
        (Entity, &Transform, &Building, &OccupiedTiles, &mut Inventory),
        (With<UnbuiltBuilding>, Without<Player>, Without<Builder>),
    >,
) {
    let Ok((mut builder, mut builder_tf)) = builder_query.single_mut() else {
        return;
    };
    let Ok((player_tf, mut player_inv)) = player_query.single_mut() else {
        return;
    };

    let speed = cfg.builder_speed;
    let reach_dist = cfg.builder_reach;
    let range_sq = (cfg.builder_range_tiles * cfg.tile_size).powi(2);

    match builder.state.clone() {
        BuilderState::Idle => {
            // Move toward player when idle
            let offset = Vec3::new(cfg.builder_idle_offset_x, cfg.builder_idle_offset_y, 0.0);
            let target = player_tf.translation + offset;
            let d = target - builder_tf.translation;
            let dist = d.length();
            if dist > 4.0 {
                move_toward(&mut builder_tf.translation, target, speed, time.delta_secs());
            }

            // Find closest unbuilt within range
            let mut closest: Option<(Entity, f32)> = None;
            for (build_entity, _tf, building, footprint, _inv) in building_query.iter() {
                let in_range = footprint.0.iter().any(|(tx, ty)| {
                    let pos = tile_to_world(*tx, *ty, cfg.tile_size);
                    let (wx, wy) = (pos.x, pos.y);
                    let dx = player_tf.translation.x - wx;
                    let dy = player_tf.translation.y - wy;
                    dx * dx + dy * dy <= range_sq
                });
                if !in_range {
                    continue;
                }
                let def = match registry.get(&building.kind) {
                    Some(d) => d,
                    None => continue,
                };
                // Check if building still needs resources
                let still_needs = def.cost.iter().any(|c| {
                    let needed = c.amount;
                    let delivered = total_delivered_for(build_entity, c, &building_query);
                    delivered < needed
                });
                if !still_needs {
                    continue;
                }
                // Check player has at least 1 unit of a needed resource
                let has_any = def.cost.iter().any(|c| {
                    let needed = c.amount;
                    let delivered = total_delivered_for(build_entity, c, &building_query);
                    delivered < needed && player_inv.get(&c.resource) >= 1
                });
                if !has_any {
                    continue;
                }
                let foot_pos = tile_to_world(footprint.0[0].0, footprint.0[0].1, cfg.tile_size);
                let (wx, wy) = (foot_pos.x, foot_pos.y);
                let d2 = (wx - builder_tf.translation.x).powi(2)
                    + (wy - builder_tf.translation.y).powi(2);
                match closest {
                    Some((_, best_d2)) if d2 < best_d2 => closest = Some((build_entity, d2)),
                    None => closest = Some((build_entity, d2)),
                    _ => {}
                }
            }

            if let Some((target_entity, _)) = closest {
                // Pick 1 resource from player
                'pick: for (build_entity, _tf, building, _, _inv) in building_query.iter() {
                    if build_entity != target_entity {
                        continue;
                    }
                    let def = match registry.get(&building.kind) {
                        Some(d) => d,
                        None => break,
                    };
                    for c in &def.cost {
                        let delivered = total_delivered_for(build_entity, c, &building_query);
                        if delivered < c.amount && player_inv.get(&c.resource) >= 1 {
                            player_inv.remove(&c.resource, 1);
                            break 'pick;
                        }
                    }
                }
                builder.state = BuilderState::MovingToBuilding(target_entity);
            }
        }

        BuilderState::MovingToBuilding(target) => {
            // Read transform first (immutable)
            let build_pos = building_query
                .get(target)
                .ok()
                .map(|(_, tf, _, _, _)| tf.translation);
            let Some(build_pos) = build_pos else {
                builder.state = BuilderState::Idle;
                return;
            };
            let d = build_pos - builder_tf.translation;
            let dist = d.length();
            if dist <= reach_dist {
                // Determine which resource to deposit (read cost)
                let cost = building_query
                    .get(target)
                    .ok()
                    .and_then(|(_, _, building, _, inv)| {
                        let def = registry.get(&building.kind)?;
                        for c in &def.cost {
                            if inv.get(&c.resource) < c.amount {
                                return Some(c.resource.clone());
                            }
                        }
                        None
                    });
                // Deposit 1 unit (mutable)
                if let Some(resource) = cost {
                    if let Ok((_, _, _, _, mut build_inv)) =
                        building_query.get_mut(target)
                    {
                        build_inv.add(&resource, 1);
                    }
                }
                builder.state = BuilderState::ReturningToPlayer;
            } else {
                let step = (speed * time.delta_secs()).min(dist);
                builder_tf.translation += d / dist * step;
            }
        }

        BuilderState::ReturningToPlayer => {
            let offset = Vec3::new(cfg.builder_idle_offset_x, cfg.builder_idle_offset_y, 0.0);
            let target = player_tf.translation + offset;
            let d = target - builder_tf.translation;
            let dist = d.length();
            if dist <= reach_dist {
                builder.state = BuilderState::Idle;
            } else {
                move_toward(&mut builder_tf.translation, target, speed, time.delta_secs());
            }
        }
    }
}

/// Count how many units of a cost item have been delivered to a building's inventory.
fn total_delivered_for(
    entity: Entity,
    cost: &Cost,
    query: &Query<
        (Entity, &Transform, &Building, &OccupiedTiles, &mut Inventory),
        (
            With<UnbuiltBuilding>,
            Without<Player>,
            Without<Builder>,
        ),
    >,
) -> u32 {
    if let Ok((_, _, _, _, inv)) = query.get(entity) {
        inv.get(&cost.resource)
    } else {
        0
    }
}

pub fn finish_construction(
    mut commands: Commands,
    registry: Res<BuildingRegistry>,
    mut building_query: Query<
        (Entity, &Building, &mut Inventory),
        (With<UnbuiltBuilding>, Without<Player>, Without<Builder>),
    >,
) {
    for (entity, building, mut inv) in building_query.iter_mut() {
        let Some(def) = registry.get(&building.kind) else {
            continue;
        };
        let complete = def.cost.iter().all(|c| inv.get(&c.resource) >= c.amount);
        if !complete {
            continue;
        }
        // Consume construction resources
        inv.resources.clear();
        // Remove ghost visual so attach_building_visuals adds the normal sprite next frame
        commands.entity(entity).remove::<Sprite>();
        commands.entity(entity).remove::<UnbuiltBuilding>();
        commands.entity(entity).insert(Active(true));
    }
}

// ── Player position (shared resource for UI systems) ──

#[derive(Resource, Default)]
pub struct PlayerWorldPos(pub Vec3);

// ── Inventory panel (replaced by UI version in economy/ui.rs) ──

// ── Player mining ──

pub fn player_mine(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&TilePosition, &mut Inventory), With<Player>>,
    deposits: Query<(Entity, &ResourceDeposit, &TilePosition)>,
    cfg: Res<MapConfig>,
    mut chunk_grid: ResMut<crate::map::tile_grid::ChunkGrid>,
    mut commands: Commands,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok((player_tile, mut inv)) = player_query.single_mut() else {
        return;
    };

    let check_tiles = [
        (0, 0),
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
    ];

    for (dep_entity, deposit, dep_tile) in deposits.iter() {
        if deposit.amount == 0 {
            continue;
        }
        let adjacent = check_tiles
            .iter()
            .any(|&(dx, dy)| dep_tile.x == player_tile.x + dx && dep_tile.y == player_tile.y + dy);
        if !adjacent {
            continue;
        }

        inv.add(&ResourceId(deposit.resource.clone()), 1);

        if !cfg.infinite_deposits {
            use crate::map::tile_grid::CHUNK_SIZE;
            let cx = dep_tile.x.div_euclid(CHUNK_SIZE as i32);
            let cy = dep_tile.y.div_euclid(CHUNK_SIZE as i32);
            let dx = dep_tile.x.rem_euclid(CHUNK_SIZE as i32) as u32;
            let dy = dep_tile.y.rem_euclid(CHUNK_SIZE as i32) as u32;
            chunk_grid.set_deposit_amount(cx, cy, dx, dy, deposit.amount - 1);
            commands.entity(dep_entity).insert(ResourceDeposit {
                resource: deposit.resource.clone(),
                amount: deposit.amount - 1,
            });
        }
        return; // Mine one deposit per press
    }
}

pub fn camera_follow_player(
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(player_tf) = player_query.single() else {
        return;
    };
    let Ok(mut camera_tf) = camera.single_mut() else {
        return;
    };
    camera_tf.translation.x = player_tf.translation.x;
    camera_tf.translation.y = player_tf.translation.y;
}
