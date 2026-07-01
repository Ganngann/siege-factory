use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use crate::combat::Projectile;
use crate::core::game_state::GameState;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{HQ, Turret};
use crate::economy::resource::Inventory;
use crate::enemy::components::{Enemy, Health};
use crate::enemy::registry::EnemyRegistry;
use crate::events::DespawnEnemy;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::{material_from_color, ShapeCache};

pub fn enemies_damage_hq(
    enemies: Query<(Entity, &TilePosition), With<Enemy>>,
    mut hq: Query<(&mut Health, &mut Inventory), With<HQ>>,
    mut next_state: ResMut<NextState<GameState>>,
    enemies_registry: Res<EnemyRegistry>,
    cfg: Res<MapConfig>,
    mut enemy_events: EventWriter<DespawnEnemy>,
) {
    let enemy_damage = enemies_registry.get("runner")
        .map(|d| d.damage)
        .unwrap_or(10);

    let (mut hq_health, _inv) = match hq.get_single_mut() {
        Ok(h) => h,
        Err(_) => return,
    };

    let hq_tx = cfg.width / 2;
    let hq_ty = cfg.height / 2;

    for (entity, pos) in enemies.iter() {
        if pos.x == hq_tx && pos.y == hq_ty {
            enemy_events.send(DespawnEnemy(entity));
            hq_health.current = hq_health.current.saturating_sub(enemy_damage);
        }
    }

    if hq_health.current == 0 {
        next_state.set(GameState::GameOver);
    }
}

pub fn turret_shoot(
    mut commands: Commands,
    mut turrets: Query<(&Transform, &mut Turret)>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    time: Res<Time>,
    buildings: Res<BuildingRegistry>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let turret_def = match buildings.get("turret") {
        Some(d) => d,
        None => return,
    };
    let combat = match &turret_def.combat {
        Some(c) => c,
        None => return,
    };
    let range_sq = combat.range.ceil() as u32 as f32;
    let damage = combat.damage;
    let fire_interval = combat.fire_rate_sec;

    for (turret_pos, mut turret) in turrets.iter_mut() {
        turret.fire_timer += time.delta_seconds();
        if turret.fire_timer < fire_interval {
            continue;
        }

        let mut target = None;
        let mut closest_dist = range_sq;

        for (entity, enemy_pos) in enemies.iter() {
            let dist = enemy_pos.translation.distance_squared(turret_pos.translation);
            if dist < closest_dist {
                closest_dist = dist;
                target = Some(entity);
            }
        }

        if let Some(entity) = target {
            turret.fire_timer -= fire_interval;
            commands.spawn((
                Projectile {
                    target: entity,
                    speed: 300.0,
                    damage,
                },
                ColorMesh2dBundle {
                    mesh: Mesh2dHandle(shapes.circle.clone()),
                    material: material_from_color(&mut materials, Color::srgb(1.0, 0.8, 0.2)),
                    transform: Transform::from_translation(turret_pos.translation).with_scale(Vec3::splat(0.3)),
                    ..default()
                },
            ));
        }
    }
}
