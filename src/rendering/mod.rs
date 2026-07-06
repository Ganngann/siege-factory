pub mod cache;
pub mod config;
pub mod hud;
pub mod visuals;

pub use cache::*;
pub use config::*;
pub use hud::*;
pub use visuals::*;

use crate::core::game_state::GameState;
use bevy::prelude::*;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VisualsConfig::load());
        app.init_resource::<ShapeCache>();
        app.init_resource::<PreviewMaterials>();
        app.add_systems(Startup, setup_texture_cache);
        app.add_systems(Update, (tile_highlight, ensure_hp_bars, update_hp_bars));
        app.add_observer(spawn_projectile_visual);
        app.add_systems(
            Update,
            (
                sync_belt_slot_sprites,
                attach_enemy_visuals,
                attach_building_visuals,
                attach_unit_visuals,
                animate_belt_positions,
                wave_counter_ui,
                fps_overlay,
            ),
        );
        app.add_systems(OnEnter(GameState::GameOver), spawn_game_over_ui);
        app.add_systems(OnExit(GameState::GameOver), despawn_game_over_ui);
    }
}
