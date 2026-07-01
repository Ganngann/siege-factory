use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use crate::combat::Projectile;
use crate::core::game_state::GameState;
use crate::economy::unit_config::UnitConfig;
use crate::economy::systems::{HQ, OreDeposit};
use crate::economy::resource::{ResourceId, Inventory};
use crate::enemy::{Health, Enemy as EnemyComponent};
use crate::events::DespawnDeposit;
use crate::map::config::MapConfig;
use crate::rendering::{material_from_color, ShapeCache};

#[derive(Event)]
pub struct SpawnUnitEvent(pub UnitKind);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    Soldier,
    Worker,
}

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnUnitEvent>();
        app.add_systems(Update, (
            spawn_unit_input,
            soldier_auto_attack,
            worker_harvest,
        ).run_if(in_state(GameState::Playing)));
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

fn spawn_unit_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    unit_cfg: Res<UnitConfig>,
    mut hq_query: Query<(&Transform, &mut Inventory), With<HQ>>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut spawn_events: EventReader<SpawnUnitEvent>,
) {
    let (hq_transform, mut inv) = match hq_query.get_single_mut() {
        Ok(q) => q,
        Err(_) => return,
    };

    let soldier_cost = unit_cfg.soldier.unit.cost.iter()
        .find(|c| c.resource == ResourceId::Ore)
        .map(|c| c.amount)
        .unwrap_or(10);
    let soldier_hp = unit_cfg.soldier.unit.hp;
    let soldier_color = unit_cfg.soldier.unit.color;

    if keys.just_pressed(KeyCode::Digit6)
        && inv.get(ResourceId::Ore) >= soldier_cost {
        inv.remove(ResourceId::Ore, soldier_cost);
        commands.spawn((
            Soldier { attack_cooldown: 0.0 },
            Health { current: soldier_hp, max: soldier_hp },
            ColorMesh2dBundle {
                mesh: Mesh2dHandle(shapes.pentagon.clone()),
                material: material_from_color(&mut materials, soldier_color),
                transform: Transform::from_xyz(
                    hq_transform.translation.x + 40.0,
                    hq_transform.translation.y,
                    2.5,
                ),
                ..default()
            },
        ));
    }

    let worker_cost = unit_cfg.worker.unit.cost.iter()
        .find(|c| c.resource == ResourceId::Ore)
        .map(|c| c.amount)
        .unwrap_or(5);
    let worker_hp = unit_cfg.worker.unit.hp;
    let worker_color = unit_cfg.worker.unit.color;

    if keys.just_pressed(KeyCode::Digit7)
        && inv.get(ResourceId::Ore) >= worker_cost {
        inv.remove(ResourceId::Ore, worker_cost);
        commands.spawn((
            Worker { state: WorkerState::Idle, mining_timer: 0.0 },
            Health { current: worker_hp, max: worker_hp },
            ColorMesh2dBundle {
                mesh: Mesh2dHandle(shapes.circle.clone()),
                material: material_from_color(&mut materials, worker_color),
                transform: Transform::from_xyz(
                    hq_transform.translation.x - 40.0,
                    hq_transform.translation.y,
                    2.5,
                ),
                ..default()
            },
        ));
    }

    for ev in spawn_events.read() {
        match ev.0 {
            UnitKind::Soldier => {
                if inv.get(ResourceId::Ore) >= soldier_cost {
                    inv.remove(ResourceId::Ore, soldier_cost);
                    commands.spawn((
                        Soldier { attack_cooldown: 0.0 },
                        Health { current: soldier_hp, max: soldier_hp },
                        ColorMesh2dBundle {
                            mesh: Mesh2dHandle(shapes.pentagon.clone()),
                            material: material_from_color(&mut materials, soldier_color),
                            transform: Transform::from_xyz(
                                hq_transform.translation.x + 40.0,
                                hq_transform.translation.y,
                                2.5,
                            ),
                            ..default()
                        },
                    ));
                }
            }
            UnitKind::Worker => {
                if inv.get(ResourceId::Ore) >= worker_cost {
                    inv.remove(ResourceId::Ore, worker_cost);
                    commands.spawn((
                        Worker { state: WorkerState::Idle, mining_timer: 0.0 },
                        Health { current: worker_hp, max: worker_hp },
                        ColorMesh2dBundle {
                            mesh: Mesh2dHandle(shapes.circle.clone()),
                            material: material_from_color(&mut materials, worker_color),
                            transform: Transform::from_xyz(
                                hq_transform.translation.x - 40.0,
                                hq_transform.translation.y,
                                2.5,
                            ),
                            ..default()
                        },
                    ));
                }
            }
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
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let range_sq = (unit_cfg.soldier.range_tiles * cfg.tile_size)
        * (unit_cfg.soldier.range_tiles * cfg.tile_size);
    let damage = unit_cfg.soldier.damage;
    let fire_rate = unit_cfg.soldier.fire_rate_sec;

    for (soldier_pos, mut soldier) in soldiers.iter_mut() {
        soldier.attack_cooldown -= time.delta_seconds();
        if soldier.attack_cooldown > 0.0 {
            continue;
        }

        let mut target = None;
        let mut closest = range_sq;

        for (enemy_entity, enemy_pos) in enemies.iter() {
            let dist = enemy_pos.translation.distance_squared(soldier_pos.translation);
            if dist < closest {
                closest = dist;
                target = Some(enemy_entity);
            }
        }

        if let Some(enemy_entity) = target {
            soldier.attack_cooldown = fire_rate;
            commands.spawn((
                Projectile {
                    target: enemy_entity,
                    speed: 300.0,
                    damage,
                },
                ColorMesh2dBundle {
                    mesh: Mesh2dHandle(shapes.circle.clone()),
                    material: material_from_color(&mut materials, Color::srgb(0.3, 1.0, 0.3)),
                    transform: Transform::from_translation(soldier_pos.translation).with_scale(Vec3::splat(0.3)),
                    ..default()
                },
            ));
        }
    }
}

fn worker_harvest(
    time: Res<Time>,
    unit_cfg: Res<UnitConfig>,
    cfg: Res<MapConfig>,
    mut workers: Query<(Entity, &mut Transform, &mut Worker)>,
    mut deposits: Query<(Entity, &mut OreDeposit, &Transform), Without<Worker>>,
    mut hq_query: Query<&mut Inventory, With<HQ>>,
    mut deposit_events: EventWriter<DespawnDeposit>,
) {
    let tile_size = cfg.tile_size;
    let speed = unit_cfg.worker.speed;
    let mine_interval = unit_cfg.worker.mine_interval_sec;

    let mut hq_inv = match hq_query.get_single_mut() {
        Ok(inv) => inv,
        Err(_) => return,
    };

    for (_worker_entity, mut transform, mut worker) in workers.iter_mut() {
        match worker.state.clone() {
            WorkerState::Idle => {
                let worker_pos = transform.translation;
                let mut nearest = None;
                let mut nearest_dist = f32::MAX;
                for (dep_entity, deposit, dep_transform) in deposits.iter() {
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
                if let Ok((_, deposit, dep_transform)) = deposits.get(target_dep) {
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
                        let step = (speed * time.delta_seconds()).min(dist);
                        transform.translation.x += dx / dist * step;
                        transform.translation.y += dy / dist * step;
                    }
                } else {
                    worker.state = WorkerState::Idle;
                }
            }
            WorkerState::Mining(target_dep) => {
                if let Ok((_dep_entity, mut deposit, _)) = deposits.get_mut(target_dep) {
                    if deposit.amount > 0 {
                        worker.mining_timer += time.delta_seconds();
                        while worker.mining_timer >= mine_interval && deposit.amount > 0 {
                            worker.mining_timer -= mine_interval;
                            deposit.amount -= 1;
                            hq_inv.add(ResourceId::Ore, 1);
                        }
                    }
                    if deposit.amount == 0 {
                        worker.state = WorkerState::Idle;
                    }
                } else {
                    worker.state = WorkerState::Idle;
                }
            }
        }
    }

    for (entity, deposit, _) in deposits.iter() {
        if deposit.amount == 0 {
            deposit_events.send(DespawnDeposit(entity));
        }
    }
}
