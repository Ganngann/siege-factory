use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[derive(Component)]
pub struct Enemy {
    pub kind: String,
}

use crate::enemy::wave_config::WaveEntry;

#[derive(Resource)]
pub struct WaveState {
    pub timer: f32,
    pub wave: u32,
    pub spawn_timer: f32,
    pub spawn_queue: Vec<WaveEntry>,
}

impl WaveState {
    pub fn total_remaining(&self) -> u32 {
        self.spawn_queue.iter().map(|e| e.count).sum()
    }
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            timer: 3.0,
            wave: 1,
            spawn_timer: 0.0,
            spawn_queue: Vec::new(),
        }
    }
}

#[derive(Component)]
pub struct WaveCounterText;

#[derive(Component)]
pub struct GameOverUi;

#[derive(Resource)]
pub struct LastWave(pub u32);
