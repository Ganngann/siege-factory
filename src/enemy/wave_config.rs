use bevy::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Resource)]
pub struct WaveConfig {
    pub win_waves: u32,
    pub wave_interval_sec: f32,
    pub spawn_interval_sec: f32,
    pub spawn_timer_min: f32,
    pub hp_per_wave: u32,
    pub max_enemies_base: u32,
    pub max_enemies_cap: u32,
}

impl WaveConfig {
    pub fn load() -> Self {
        let toml_str = include_str!("../../data/waves.toml");
        let parsed: WavesToml = toml::from_str(toml_str).expect("failed to parse waves.toml");
        Self {
            win_waves: parsed.game.win_waves,
            wave_interval_sec: parsed.game.wave_interval_sec,
            spawn_interval_sec: parsed.game.spawn_interval_sec,
            spawn_timer_min: parsed.game.spawn_timer_min,
            hp_per_wave: parsed.game.hp_per_wave,
            max_enemies_base: parsed.game.max_enemies_base,
            max_enemies_cap: parsed.game.max_enemies_cap,
        }
    }
}

#[derive(Deserialize)]
struct WavesToml {
    game: GameEntry,
}

#[derive(Deserialize)]
struct GameEntry {
    win_waves: u32,
    wave_interval_sec: f32,
    spawn_interval_sec: f32,
    spawn_timer_min: f32,
    hp_per_wave: u32,
    max_enemies_base: u32,
    max_enemies_cap: u32,
}
