use bevy::prelude::*;

use crate::core::game_state::GameState;
use crate::core::utils::move_toward;
use crate::economy::components::{Player, ResourceDeposit, Unit};
use crate::economy::resource::{Inventory, ResourceId};
use crate::economy::unit_config::UnitConfig;
use crate::enemy::combat::find_closest_enemy;
use crate::enemy::{Enemy as EnemyComponent, Health};
use crate::events::{DespawnDeposit, SpawnProjectileEvent};
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::map::tile_grid::{CHUNK_SIZE, ChunkGrid};

const UNIT_KIND_HARVESTER: &str = "harvester";

#[derive(Event)]
pub struct SpawnUnitEvent(pub String);

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_unit_on_trigger);
        app.add_systems(
            Update,
            (soldier_auto_attack, worker_harvest).run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Soldier {
    pub attack_cooldown: f32,
}

#[derive(Component)]
pub struct Worker {
    pub state: WorkerState,
    pub mining_timer: f32,
}

#[derive(Clone)]
pub enum WorkerState {
    Idle,
    MovingToDeposit(Entity),
    Mining(Entity),
}

fn spawn_unit_by_id(
    commands: &mut Commands,
    unit_cfg: &UnitConfig,
    id: &str,
    hq_pos: Vec3,
) -> bool {
    let def = match unit_cfg.get(id) {
        Some(d) => d,
        None => return false,
    };
    let hp = def.hp;
    let offset = if def.kind == UNIT_KIND_HARVESTER {
        Vec3::new(-40.0, 0.0, 2.5)
    } else {
        Vec3::new(40.0, 0.0, 2.5)
    };

    if def.kind == UNIT_KIND_HARVESTER {
        commands.spawn((
            Worker {
                state: WorkerState::Idle,
                mining_timer: 0.0,
            },
            Unit,
            Health {
                current: hp,
                max: hp,
            },
            Transform::from_translation(hq_pos + offset),
        ));
    } else {
        commands.spawn((
            Soldier {
                attack_cooldown: 0.0,
            },
            Unit,
            Health {
                current: hp,
                max: hp,
            },
            Transform::from_translation(hq_pos + offset),
        ));
    }
    true
}

fn spawn_unit_on_trigger(
    on: On<SpawnUnitEvent>,
    unit_cfg: Res<UnitConfig>,
    mut player_query: Query<(&Transform, &mut Inventory), With<Player>>,
    mut commands: Commands,
) {
    let (player_transform, mut inv) = match player_query.single_mut() {
        Ok(q) => q,
        Err(_) => return,
    };
    let id = &on.event().0;
    if let Some(def) = unit_cfg.get(id) {
        let can_afford = def.cost.iter().all(|c| inv.get(&c.resource) >= c.amount);
        if can_afford {
            for c in &def.cost {
                inv.remove(&c.resource, c.amount);
            }
            spawn_unit_by_id(&mut commands, &unit_cfg, id, player_transform.translation);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn soldier_auto_attack(
    mut commands: Commands,
    mut soldiers: Query<(&Transform, &mut Soldier)>,
    enemies: Query<(Entity, &Transform), With<EnemyComponent>>,
    time: Res<Time>,
    unit_cfg: Res<UnitConfig>,
    cfg: Res<MapConfig>,
) {
    let soldier_def = match unit_cfg.get("soldier") {
        Some(d) => d,
        None => return,
    };
    let range_sq =
        (soldier_def.range_tiles * cfg.tile_size) * (soldier_def.range_tiles * cfg.tile_size);
    let damage = soldier_def.damage;
    let fire_rate = soldier_def.fire_rate_sec;

    for (soldier_pos, mut soldier) in soldiers.iter_mut() {
        soldier.attack_cooldown -= time.delta_secs();
        if soldier.attack_cooldown > 0.0 {
            continue;
        }

        let enemy_positions: Vec<(Entity, Vec3)> =
            enemies.iter().map(|(e, t)| (e, t.translation)).collect();

        let target = find_closest_enemy(soldier_pos.translation, &enemy_positions, range_sq);

        if let Some(enemy_entity) = target {
            soldier.attack_cooldown = fire_rate;
            commands.trigger(SpawnProjectileEvent {
                target: enemy_entity,
                speed: soldier_def.projectile_speed,
                damage,
                origin: soldier_pos.translation,
                color: Color::srgb(0.3, 1.0, 0.3),
            });
        }
    }
}

fn worker_harvest(
    time: Res<Time>,
    unit_cfg: Res<UnitConfig>,
    cfg: Res<MapConfig>,
    mut chunk_grid: ResMut<ChunkGrid>,
    mut workers: Query<(Entity, &mut Transform, &mut Worker)>,
    mut deposits: Query<(Entity, &mut ResourceDeposit, &Transform, &TilePosition), Without<Worker>>,
    mut player_query: Query<&mut Inventory, With<Player>>,
    mut commands: Commands,
) {
    let tile_size = cfg.tile_size;
    let worker_def = match unit_cfg.get("worker") {
        Some(d) => d,
        None => return,
    };
    let speed = worker_def.speed;
    let mine_interval = worker_def.mine_interval_sec;

    let mut player_inv = match player_query.single_mut() {
        Ok(inv) => inv,
        Err(_) => return,
    };

    for (_worker_entity, mut transform, mut worker) in workers.iter_mut() {
        match worker.state.clone() {
            WorkerState::Idle => {
                let worker_pos = transform.translation;
                let mut nearest = None;
                let mut nearest_dist = f32::MAX;
                for (dep_entity, deposit, dep_transform, _) in deposits.iter() {
                    if deposit.amount > 0 {
                        let dist = worker_pos.distance(dep_transform.translation);
                        if dist < nearest_dist {
                            nearest_dist = dist;
                            nearest = Some(dep_entity);
                        }
                    }
                }
                if let Some(dep_entity) = nearest {
                    worker.state = WorkerState::MovingToDeposit(dep_entity);
                }
            }
            WorkerState::MovingToDeposit(target_dep) => {
                if let Ok((_, deposit, dep_transform, _)) = deposits.get(target_dep) {
                    if deposit.amount == 0 {
                        worker.state = WorkerState::Idle;
                        continue;
                    }
                    let target_pos = dep_transform.translation;
                    let dx = target_pos.x - transform.translation.x;
                    let dy = target_pos.y - transform.translation.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < tile_size * 0.5 {
                        worker.state = WorkerState::Mining(target_dep);
                    } else {
                        move_toward(&mut transform.translation, target_pos, speed, time.delta_secs());
                    }
                } else {
                    worker.state = WorkerState::Idle;
                }
            }
            WorkerState::Mining(target_dep) => {
                if let Ok((_dep_entity, mut deposit, _, _)) = deposits.get_mut(target_dep) {
                    if deposit.amount > 0 || cfg.infinite_deposits {
                        worker.mining_timer += time.delta_secs();
                        while worker.mining_timer >= mine_interval
                            && (deposit.amount > 0 || cfg.infinite_deposits)
                        {
                            worker.mining_timer -= mine_interval;
                            if !cfg.infinite_deposits {
                                deposit.amount = deposit.amount.saturating_sub(1);
                            }
                            player_inv.add(&ResourceId(deposit.resource.clone()), 1);
                        }
                    }
                    if !cfg.infinite_deposits && deposit.amount == 0 {
                        worker.state = WorkerState::Idle;
                    }
                } else {
                    worker.state = WorkerState::Idle;
                }
            }
        }
    }

    if !cfg.infinite_deposits {
        for (entity, deposit, _, tile_pos) in deposits.iter() {
            if deposit.amount == 0 {
                let cx = tile_pos.x.div_euclid(CHUNK_SIZE as i32);
                let cy = tile_pos.y.div_euclid(CHUNK_SIZE as i32);
                let dx = tile_pos.x.rem_euclid(CHUNK_SIZE as i32) as u32;
                let dy = tile_pos.y.rem_euclid(CHUNK_SIZE as i32) as u32;
                chunk_grid.set_deposit_amount(cx, cy, dx, dy, 0);
                commands.trigger(DespawnDeposit(entity));
            }
        }
    }
}
