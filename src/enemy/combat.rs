use bevy::prelude::*;

use crate::economy::components::{Player, PowerConsumer, TurretCombat, UnbuiltBuilding};
use crate::enemy::components::{Enemy, Health};
use crate::enemy::registry::EnemyRegistry;
use crate::events::{DespawnEnemy, SpawnProjectileEvent};
use crate::map::components::TilePosition;

/// Find the closest enemy entity within range_sq of a given position.
pub fn find_closest_enemy(pos: Vec3, enemies: &[(Entity, Vec3)], range_sq: f32) -> Option<Entity> {
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

pub fn enemies_damage_player(
    enemies: Query<(Entity, &Enemy, &TilePosition)>,
    mut player_query: Query<(&mut Health, &TilePosition), With<Player>>,
    enemies_registry: Res<EnemyRegistry>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<crate::core::game_state::GameState>>,
) {
    let Ok((mut player_health, player_pos)) = player_query.single_mut() else {
        return;
    };

    for (entity, enemy, pos) in enemies.iter() {
        if pos.x == player_pos.x && pos.y == player_pos.y {
            let damage = enemies_registry
                .get(&enemy.kind)
                .map(|d| d.damage)
                .unwrap_or(10);
            commands.trigger(DespawnEnemy(entity));
            player_health.current = player_health.current.saturating_sub(damage);
            if player_health.current == 0 {
                next_state.set(crate::core::game_state::GameState::GameOver);
                return;
            }
        }
    }
}

pub fn turret_shoot(
    mut commands: Commands,
    mut turrets: Query<
        (&Transform, &mut TurretCombat, Option<&PowerConsumer>),
        Without<UnbuiltBuilding>,
    >,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    time: Res<Time>,
) {
    let enemy_positions: Vec<(Entity, Vec3)> =
        enemies.iter().map(|(e, t)| (e, t.translation)).collect();

    for (turret_pos, mut combat, power) in turrets.iter_mut() {
        if let Some(pc) = power {
            if !pc.satisfied {
                continue;
            }
        }
        combat.timer += time.delta_secs();
        if combat.timer < combat.fire_interval {
            continue;
        }

        if let Some(entity) =
            find_closest_enemy(turret_pos.translation, &enemy_positions, combat.range_sq)
        {
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
