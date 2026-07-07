pub mod cache;
pub mod config;
pub mod hud;
pub mod minimap;
pub mod power_lines;
pub mod visuals;

pub use cache::*;
pub use config::*;
pub use hud::*;
pub use visuals::*;

use crate::core::game_state::GameState;
use bevy::prelude::*;

fn cleanup_tile_highlight(mut commands: Commands, mut highlight: ResMut<TileHighlightEntity>) {
    if let Some(entity) = highlight.0.take() {
        commands.entity(entity).despawn();
    }
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VisualsConfig::load());
        app.init_resource::<ShapeCache>();
        app.init_resource::<PreviewMaterials>();
        app.init_resource::<TileHighlightEntity>();
        app.add_systems(Startup, setup_texture_cache);
        app.insert_resource(FpsUpdateTimer::default());
        app.add_systems(
            Update,
            (
                tile_highlight,
                ensure_hp_bars,
                update_hp_bars,
                ensure_mining_progress_bars,
                update_mining_progress_bars,
            )
                .run_if(in_state(GameState::Playing)),
        );
        app.add_observer(spawn_projectile_visual);
        app.add_systems(
            Update,
            (
                sync_belt_slot_sprites,
                attach_enemy_visuals,
                attach_building_visuals,
                attach_unit_visuals,
                animate_belt_positions,
                update_tier_visuals,
                wave_counter_ui,
                power_lines::render_power_lines,
            )
                .run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update, fps_overlay.run_if(in_state(GameState::Playing)));
        app.add_systems(OnEnter(GameState::GameOver), spawn_game_over_ui);
        app.add_systems(OnExit(GameState::GameOver), despawn_game_over_ui);
        app.add_systems(OnEnter(GameState::Playing), minimap::setup_minimap);
        app.add_systems(
            Update,
            minimap::update_minimap.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(OnExit(GameState::Playing), cleanup_tile_highlight);
    }
}
