use crate::core::game_state::GameState;
use crate::core::utils::move_toward;
use crate::enemy::wave_config::WaveConfig;
use crate::enemy::{Enemy as EnemyComponent, Health};
use crate::events::DespawnEnemy;
use bevy::prelude::*;

#[derive(Component)]
pub struct Projectile {
    pub target: Entity,
    pub speed: f32,
    pub damage: u32,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            move_and_hit_projectiles.run_if(in_state(GameState::Playing)),
        );
    }
}

fn move_and_hit_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    wave_cfg: Res<WaveConfig>,
    mut projectiles: Query<(Entity, &mut Transform, &Projectile), Without<EnemyComponent>>,
    mut targets: Query<(&mut Health, &Transform), (With<EnemyComponent>, Without<Projectile>)>,
) {
    let hit_dist = wave_cfg.projectile_hit_distance;
    let mut to_despawn = Vec::new();

    for (proj_entity, mut transform, projectile) in projectiles.iter_mut() {
        if let Ok((mut health, target_transform)) = targets.get_mut(projectile.target) {
            let dir = target_transform.translation - transform.translation;
            let dist = dir.length();

            if dist < hit_dist {
                health.current = health.current.saturating_sub(projectile.damage);
                if health.current == 0 {
                    commands.trigger(DespawnEnemy(projectile.target));
                }
                to_despawn.push(proj_entity);
            } else {
                move_toward(&mut transform.translation, target_transform.translation, projectile.speed, time.delta_secs());
            }
        } else {
            to_despawn.push(proj_entity);
        }
    }

    for entity in to_despawn {
        commands.entity(entity).despawn();
    }
}
