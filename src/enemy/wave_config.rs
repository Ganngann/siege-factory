use crate::load_toml;
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct WaveEntry {
    pub kind: String,
    pub count: u32,
}

#[derive(Debug, Clone)]
pub struct WaveDef {
    pub enemies: Vec<WaveEntry>,
}

#[derive(Debug, Clone, Resource)]
pub struct WaveConfig {
    pub win_waves: u32,
    pub first_wave_delay: f32,
    pub wave_interval_sec: f32,
    pub spawn_interval_sec: f32,
    pub spawn_timer_min: f32,
    pub hp_per_wave: u32,
    pub max_enemies_base: u32,
    pub max_enemies_cap: u32,
    pub projectile_hit_distance: f32,
    pub spawn_distance: f32,
    pub enemy_arrival_threshold: f32,
    pub enemy_spawn_z: f32,
    pub waves: Vec<WaveDef>,
}

impl WaveConfig {
    pub fn load() -> Self {
        let parsed: WavesToml = load_toml!("../../data/waves.toml", WavesToml);
        let waves = parsed
            .waves
            .iter()
            .map(|w| WaveDef {
                enemies: w
                    .enemies
                    .iter()
                    .map(|e| WaveEntry {
                        kind: e.kind.clone(),
                        count: e.count,
                    })
                    .collect(),
            })
            .collect();
        Self {
            win_waves: parsed.game.win_waves,
            first_wave_delay: parsed.game.first_wave_delay,
            wave_interval_sec: parsed.game.wave_interval_sec,
            spawn_interval_sec: parsed.game.spawn_interval_sec,
            spawn_timer_min: parsed.game.spawn_timer_min,
            hp_per_wave: parsed.game.hp_per_wave,
            max_enemies_base: parsed.game.max_enemies_base,
            max_enemies_cap: parsed.game.max_enemies_cap,
            projectile_hit_distance: parsed.game.projectile_hit_distance,
            spawn_distance: parsed.game.spawn_distance,
            enemy_arrival_threshold: parsed.game.enemy_arrival_threshold,
            enemy_spawn_z: parsed.game.enemy_spawn_z,
            waves,
        }
    }
}

#[derive(Deserialize)]
struct WavesToml {
    game: GameEntry,
    waves: Vec<WaveTomlEntry>,
}

#[derive(Deserialize)]
struct WaveTomlEntry {
    enemies: Vec<EnemyTomlEntry>,
}

#[derive(Deserialize)]
struct EnemyTomlEntry {
    kind: String,
    count: u32,
}

#[derive(Deserialize)]
struct GameEntry {
    win_waves: u32,
    first_wave_delay: f32,
    wave_interval_sec: f32,
    spawn_interval_sec: f32,
    spawn_timer_min: f32,
    hp_per_wave: u32,
    max_enemies_base: u32,
    max_enemies_cap: u32,
    #[serde(default = "default_hit_distance")]
    projectile_hit_distance: f32,
    #[serde(default = "default_spawn_distance")]
    spawn_distance: f32,
    #[serde(default = "default_enemy_arrival_threshold")]
    enemy_arrival_threshold: f32,
    #[serde(default = "default_enemy_spawn_z")]
    enemy_spawn_z: f32,
}

fn default_hit_distance() -> f32 {
    10.0
}
fn default_spawn_distance() -> f32 {
    25.0
}
fn default_enemy_arrival_threshold() -> f32 {
    2.0
}
fn default_enemy_spawn_z() -> f32 {
    3.0
}


