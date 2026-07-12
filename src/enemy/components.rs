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
    pub fn new(first_delay: f32) -> Self {
        Self {
            timer: first_delay,
            wave: 1,
            spawn_timer: 0.0,
            spawn_queue: Vec::new(),
        }
    }

    pub fn total_remaining(&self) -> u32 {
        self.spawn_queue.iter().map(|e| e.count).sum()
    }
}

// WaveState uses WaveConfig::first_wave_delay for its initial timer,

#[derive(Resource)]
pub struct LastWave(pub u32);


