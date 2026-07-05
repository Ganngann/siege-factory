use bevy::prelude::*;

use crate::core::game_state::GameState;
use crate::economy::components::{HQ, TurretCombat};
use crate::enemy::components::{Enemy, Health};
use crate::enemy::registry::EnemyRegistry;
use crate::events::{DespawnEnemy, SpawnProjectileEvent};
use crate::map::components::TilePosition;

/// Find the closest enemy entity within range_sq of a given position.
pub fn find_closest_enemy(
    pos: Vec3,
    enemies: &[(Entity, Vec3)],
    range_sq: f32,
) -> Option<Entity> {
    let mut target = None;
    let mut closest_dist = range_sq;

    for (entity, enemy_pos) in enemies {
        let dist = enemy_pos.distance_squared(pos);
        if dist < closest_dist {
            closest_dist = dist;
            target = Some(*entity);
        }
    }

    target
}

pub fn enemies_damage_hq(
    enemies: Query<(Entity, &Enemy, &TilePosition)>,
    mut hq: Query<(&mut Health, &TilePosition), With<HQ>>,
    mut next_state: ResMut<NextState<GameState>>,
    enemies_registry: Res<EnemyRegistry>,
    mut commands: Commands,
) {
    let (mut hq_health, hq_pos) = match hq.single_mut() {
        Ok(h) => h,
        Err(_) => return,
    };

    for (entity, enemy, pos) in enemies.iter() {
        if pos.x == hq_pos.x && pos.y == hq_pos.y {
            let damage = enemies_registry.get(&enemy.kind)
                .map(|d| d.damage)
                .unwrap_or(10);
            commands.trigger(DespawnEnemy(entity));
            hq_health.current = hq_health.current.saturating_sub(damage);
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
            commands.trigger(SpawnProjectileEvent {
                target: entity,
                speed: combat.projectile_speed,
                damage: combat.damage,
                origin: turret_pos.translation,
                color: Color::srgb(1.0, 0.8, 0.2),
            });
        }
    }
}
