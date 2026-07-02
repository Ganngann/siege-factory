use bevy::prelude::*;

use crate::combat::Projectile;
use crate::core::game_state::GameState;
use crate::economy::unit_config::UnitConfig;
use crate::economy::components::{HQ, OreDeposit, Unit};
use crate::economy::resource::{ResourceId, Inventory};
use crate::enemy::{Health, Enemy as EnemyComponent};
use crate::events::DespawnDeposit;
use crate::map::config::MapConfig;
use crate::rendering::ShapeCache;


#[derive(Event)]
pub struct SpawnUnitEvent(pub String);

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_unit_on_trigger);
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

fn spawn_unit_by_id(
    commands: &mut Commands,
    unit_cfg: &UnitConfig,
    id: &str,
    hq_pos: Vec3,
    shapes: &ShapeCache,
    materials: &mut Assets<ColorMaterial>,
) -> bool {
    let def = match unit_cfg.get(id) {
        Some(d) => d,
        None => return false,
    };
    let hp = def.hp;
    let color = def.color;
    let mesh_name = &def.visual;
    let mesh_handle = match mesh_name.as_str() {
        "pentagon" => shapes.pentagon.clone(),
        "circle" => shapes.circle.clone(),
        _ => shapes.pentagon.clone(),
    };
    let offset = if def.kind == "harvester" {
        Vec3::new(-40.0, 0.0, 2.5)
    } else {
        Vec3::new(40.0, 0.0, 2.5)
    };
    let mesh_mat = MeshMaterial2d(materials.add(color));
    if def.kind == "harvester" {
        commands.spawn((
            Worker { state: WorkerState::Idle, mining_timer: 0.0 },
            Unit, Health { current: hp, max: hp },
            Mesh2d(mesh_handle), mesh_mat,
            Transform::from_translation(hq_pos + offset),
        ));
    } else {
        commands.spawn((
            Soldier { attack_cooldown: 0.0 },
            Unit, Health { current: hp, max: hp },
            Mesh2d(mesh_handle), mesh_mat,
            Transform::from_translation(hq_pos + offset),
        ));
    }
    true
}

fn spawn_unit_on_trigger(
    on: On<SpawnUnitEvent>,
    unit_cfg: Res<UnitConfig>,
    mut hq_query: Query<(&Transform, &mut Inventory), With<HQ>>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    let (hq_transform, mut inv) = match hq_query.single_mut() {
        Ok(q) => q,
        Err(_) => return,
    };
    let id = &on.event().0;
    if let Some(def) = unit_cfg.get(id) {
        let cost_ore = def.cost.iter()
            .find(|c| c.resource == ResourceId::Ore)
            .map(|c| c.amount)
            .unwrap_or(0);
        if inv.get(ResourceId::Ore) >= cost_ore {
            inv.remove(ResourceId::Ore, cost_ore);
            spawn_unit_by_id(&mut commands, &unit_cfg, id, hq_transform.translation, &shapes, &mut materials);
        }
    }
}

fn spawn_unit_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    unit_cfg: Res<UnitConfig>,
    mut hq_query: Query<(&Transform, &mut Inventory), With<HQ>>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let (hq_transform, mut inv) = match hq_query.single_mut() {
        Ok(q) => q,
        Err(_) => return,
    };
    let hq_pos = hq_transform.translation;

    let key_units = [
        (KeyCode::Digit6, "soldier"),
        (KeyCode::Digit7, "worker"),
    ];

    for (key, unit_id) in key_units {
        if keys.just_pressed(key) {
            if let Some(def) = unit_cfg.get(unit_id) {
                let cost_ore = def.cost.iter()
                    .find(|c| c.resource == ResourceId::Ore)
                    .map(|c| c.amount)
                    .unwrap_or(0);
                if inv.get(ResourceId::Ore) >= cost_ore {
                    inv.remove(ResourceId::Ore, cost_ore);
                    spawn_unit_by_id(&mut commands, &unit_cfg, unit_id, hq_pos, &shapes, &mut materials);
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
    let soldier_def = match unit_cfg.get("soldier") {
        Some(d) => d,
        None => return,
    };
    let range_sq = (soldier_def.range_tiles * cfg.tile_size)
        * (soldier_def.range_tiles * cfg.tile_size);
    let damage = soldier_def.damage;
    let fire_rate = soldier_def.fire_rate_sec;

    for (soldier_pos, mut soldier) in soldiers.iter_mut() {
        soldier.attack_cooldown -= time.delta_secs();
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
                Mesh2d(shapes.circle.clone()),
                MeshMaterial2d(materials.add(Color::srgb(0.3, 1.0, 0.3))),
                Transform::from_translation(soldier_pos.translation).with_scale(Vec3::splat(0.3)),
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
    mut commands: Commands,
) {
    let tile_size = cfg.tile_size;
    let worker_def = match unit_cfg.get("worker") {
        Some(d) => d,
        None => return,
    };
    let speed = worker_def.speed;
    let mine_interval = worker_def.mine_interval_sec;

    let mut hq_inv = match hq_query.single_mut() {
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
                        let step = (speed * time.delta_secs()).min(dist);
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
                        worker.mining_timer += time.delta_secs();
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
            commands.trigger(DespawnDeposit(entity));
        }
    }
}
