pub mod registry;
pub mod wave_config;
pub mod components;
pub mod ai;
pub mod combat;
pub mod systems;

pub use components::{Health, Enemy};

use bevy::prelude::*;
use crate::core::game_state::GameState;
use crate::enemy::components::{WaveState, LastWave};
use registry::EnemyRegistry;
use wave_config::WaveConfig;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WaveState::default());
        app.insert_resource(EnemyRegistry::load());
        app.insert_resource(WaveConfig::load());
        app.insert_resource(LastWave(1));
        app.add_systems(OnEnter(GameState::Playing), systems::reset_wave.run_if(crate::save_load::is_fresh_game));
        app.add_systems(OnExit(GameState::Playing), systems::cleanup_game_entities);
        app.add_systems(Update, (
            systems::wave_timer,
            systems::spawn_enemies,
            ai::move_enemies,
            combat::enemies_damage_hq,
            combat::turret_shoot,
            systems::wave_announcement,
        ).run_if(in_state(GameState::Playing)));
    }
}
