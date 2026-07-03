use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[derive(Component)]
pub struct Enemy;

#[derive(Resource)]
pub struct WaveState {
    pub timer: f32,
    pub wave: u32,
    pub spawn_timer: f32,
    pub enemies_this_wave: u32,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            timer: 3.0,
            wave: 1,
            spawn_timer: 0.0,
            enemies_this_wave: 0,
        }
    }
}

#[derive(Component)]
pub struct WaveCounterText;

#[derive(Component)]
pub struct GameOverUi;

#[derive(Resource)]
pub struct LastWave(pub u32);
