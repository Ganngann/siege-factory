use bevy::prelude::*;

use crate::core::toast::ToastQueue;
use crate::economy::components::{HQ, PeacefulMode};
use crate::enemy::components::{Enemy, Health, LastWave, WaveState};
use crate::enemy::registry::EnemyRegistry;
use crate::enemy::wave_config::WaveConfig;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;

pub fn wave_timer(
    time: Res<Time>,
    mut wave: ResMut<WaveState>,
    existing: Query<Entity, With<Enemy>>,
    cfg: Res<WaveConfig>,
    peaceful: Res<PeacefulMode>,
) {
    if peaceful.0 {
        return;
    }
    wave.timer -= time.delta_secs();
    if wave.timer <= 0.0 && existing.iter().len() == 0 {
        let wave_idx = ((wave.wave - 1) as usize).min(cfg.waves.len().saturating_sub(1));
        wave.spawn_queue = cfg.waves[wave_idx].enemies.clone();
        wave.spawn_timer = 0.0;
        wave.wave += 1;
        wave.timer = cfg.wave_interval_sec;
    }
}

pub fn spawn_enemies(
    mut commands: Commands,
    mut wave: ResMut<WaveState>,
    time: Res<Time>,
    hq: Query<&TilePosition, With<HQ>>,
    existing: Query<Entity, With<Enemy>>,
    enemies_registry: Res<EnemyRegistry>,
    cfg: Res<WaveConfig>,
    map_cfg: Res<MapConfig>,
    peaceful: Res<PeacefulMode>,
) {
    if peaceful.0 {
        return;
    }
    let tile_size = map_cfg.tile_size;

    let max_enemies = (wave.wave * cfg.max_enemies_base).min(cfg.max_enemies_cap);
    if existing.iter().len() >= max_enemies as usize {
        return;
    }

    if wave.spawn_queue.is_empty() {
        return;
    }

    wave.spawn_timer -= time.delta_secs();
    if wave.spawn_timer > 0.0 {
        return;
    }
    wave.spawn_timer = (cfg.spawn_interval_sec / wave.wave as f32).max(cfg.spawn_timer_min);

    let hq_pos = match hq.single() {
        Ok(p) => p,
        Err(_) => return,
    };

    // Spawn the next enemy type from queue
    let entry = &wave.spawn_queue[0];
    let kind = &entry.kind;
    let def = match enemies_registry.get(kind) {
        Some(d) => d,
        None => {
            wave.spawn_queue.remove(0);
            return;
        }
    };

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let spawn_dist = 25.0;
    let sx = (hq_pos.x as f32 + angle.cos() * spawn_dist).round() as i32;
    let sy = (hq_pos.y as f32 + angle.sin() * spawn_dist).round() as i32;

    let enemy_hp = def.hp + (wave.wave - 1) * cfg.hp_per_wave;

    commands.spawn((
        Enemy { kind: kind.clone() },
        Health {
            current: enemy_hp,
            max: enemy_hp,
        },
        Transform::from_xyz(
            sx as f32 * tile_size + tile_size / 2.0,
            sy as f32 * tile_size + tile_size / 2.0,
            3.0,
        ),
        TilePosition { x: sx, y: sy },
    ));

    if wave.spawn_queue[0].count > 1 {
        wave.spawn_queue[0].count -= 1;
    } else {
        wave.spawn_queue.remove(0);
    }
}

pub fn cleanup_game_entities(
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>,
    units: Query<Entity, With<crate::economy::components::Unit>>,
) {
    for entity in enemies.iter().chain(units.iter()) {
        commands.entity(entity).try_despawn();
    }
}

pub fn reset_wave(
    mut commands: Commands,
    mut wave: ResMut<WaveState>,
    hq: Query<Entity, With<HQ>>,
    cfg: Res<MapConfig>,
) {
    *wave = WaveState::default();
    if let Ok(entity) = hq.single() {
        commands.entity(entity).insert(Health {
            current: cfg.hq_hp,
            max: cfg.hq_hp,
        });
    }
}

pub fn wave_announcement(
    wave: Res<WaveState>,
    mut last_wave: ResMut<LastWave>,
    mut toast: ResMut<ToastQueue>,
) {
    if wave.wave != last_wave.0 && wave.wave > 1 {
        toast.0.push(format!("Wave {}", wave.wave));
        last_wave.0 = wave.wave;
    }
}
