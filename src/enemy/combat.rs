use bevy::prelude::*;

use crate::combat::Projectile;
use crate::core::game_state::GameState;
use crate::economy::components::{HQ, TurretCombat};
use crate::economy::resource::Inventory;
use crate::enemy::components::{Enemy, Health};
use crate::enemy::registry::EnemyRegistry;
use crate::events::DespawnEnemy;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::ShapeCache;

pub fn find_closest_enemy(
    turret_pos: Vec3,
    enemies: &[(Entity, Vec3)],
    range_sq: f32,
) -> Option<Entity> {
    let mut target = None;
    let mut closest_dist = range_sq;

    for (entity, enemy_pos) in enemies {
        let dist = enemy_pos.distance_squared(turret_pos);
        if dist < closest_dist {
            closest_dist = dist;
            target = Some(*entity);
        }
    }

    target
}

pub fn enemies_damage_hq(
    enemies: Query<(Entity, &TilePosition), With<Enemy>>,
    mut hq: Query<(&mut Health, &mut Inventory), With<HQ>>,
    mut next_state: ResMut<NextState<GameState>>,
    enemies_registry: Res<EnemyRegistry>,
    cfg: Res<MapConfig>,
    mut commands: Commands,
) {
    let enemy_damage = enemies_registry.get("runner")
        .map(|d| d.damage)
        .unwrap_or(10);

    let (mut hq_health, _inv) = match hq.single_mut() {
        Ok(h) => h,
        Err(_) => return,
    };

    let hq_tx = cfg.width / 2;
    let hq_ty = cfg.height / 2;

    for (entity, pos) in enemies.iter() {
        if pos.x == hq_tx && pos.y == hq_ty {
            commands.trigger(DespawnEnemy(entity));
            hq_health.current = hq_health.current.saturating_sub(enemy_damage);
        }
    }

    if hq_health.current == 0 {
        next_state.set(GameState::GameOver);
    }
}

pub fn turret_shoot(
    mut commands: Commands,
    mut turrets: Query<(&Transform, &mut TurretCombat)>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    time: Res<Time>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (turret_pos, mut combat) in turrets.iter_mut() {
        combat.timer += time.delta_secs();
        if combat.timer < combat.fire_interval {
            continue;
        }

        let enemy_positions: Vec<(Entity, Vec3)> = enemies.iter()
            .map(|(e, t)| (e, t.translation))
            .collect();

        if let Some(entity) = find_closest_enemy(turret_pos.translation, &enemy_positions, combat.range_sq) {
            combat.timer -= combat.fire_interval;
            commands.spawn((
                Projectile {
                    target: entity,
                    speed: 300.0,
                    damage: combat.damage,
                },
                Mesh2d(shapes.circle.clone()),
                MeshMaterial2d(materials.add(Color::srgb(1.0, 0.8, 0.2))),
                Transform::from_translation(turret_pos.translation).with_scale(Vec3::splat(0.3)),
            ));
        }
    }
}
