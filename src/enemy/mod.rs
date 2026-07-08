pub mod ai;
pub mod combat;
pub mod components;
pub mod registry;
pub mod systems;
pub mod wave_config;

pub use components::{Enemy, Health};

use crate::core::game_state::GameState;
use crate::enemy::components::{LastWave, WaveState};
use bevy::prelude::*;
use registry::EnemyRegistry;
use wave_config::WaveConfig;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        let mods = app.world().resource::<crate::core::modding::ModRegistry>().clone();
        let wave_cfg = WaveConfig::load(&mods);
        app.insert_resource(WaveState::new(wave_cfg.first_wave_delay));
        app.insert_resource(EnemyRegistry::load(&mods));
        app.insert_resource(wave_cfg);
        app.insert_resource(LastWave(1));
        app.add_systems(
            OnEnter(GameState::Playing),
            systems::reset_wave.run_if(crate::save_load::is_fresh_game),
        );
        app.add_systems(OnExit(GameState::Playing), systems::cleanup_game_entities);
        app.add_systems(
            Update,
            (
                systems::wave_timer,
                systems::spawn_enemies,
                ai::move_enemies,
                combat::enemies_damage_player,
                combat::turret_shoot,
                systems::wave_announcement,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
