pub mod cache;
pub mod config;
pub mod minimap;
pub mod power_lines;
pub mod visuals;

pub use cache::*;
pub use config::*;
pub use visuals::*;

use crate::core::game_state::GameState;
use crate::core::utils::silent_despawn;
use crate::ui::components::building_tooltip::{building_tooltip_system, TooltipTarget};
use crate::ui::components::hud_text::{update_hud_wave_counter, update_hud_fps, FpsUpdateTimer};
use crate::ui::global_panels::{spawn_game_over_overlay, despawn_game_over_overlay};
use bevy::prelude::*;

fn cleanup_tile_highlight(mut commands: Commands, mut highlight: ResMut<TileHighlightEntity>) {
    if let Some(entity) = highlight.0.take() {
        silent_despawn(&mut commands, entity);
    }
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        let mods = app.world().resource::<crate::core::modding::ModRegistry>().clone();
        app.insert_resource(VisualsConfig::load(&mods));
        app.init_resource::<ShapeCache>();
        app.init_resource::<PreviewMaterials>();
        app.init_resource::<TileHighlightEntity>();
        app.add_systems(Startup, setup_texture_cache);
        app.init_resource::<FpsUpdateTimer>();
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
                power_lines::render_power_lines,
            )
                .run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            (
                update_hud_wave_counter,
                update_hud_fps,
            )
                .run_if(in_state(GameState::Playing)),
        );
        app.add_systems(OnEnter(GameState::GameOver), spawn_game_over_overlay);
        app.add_systems(OnExit(GameState::GameOver), despawn_game_over_overlay);
        app.add_systems(OnEnter(GameState::Playing), minimap::setup_minimap);
        app.add_systems(
            Update,
            minimap::update_minimap.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(OnExit(GameState::Playing), cleanup_tile_highlight);
        app.init_resource::<TooltipTarget>();
        app.add_systems(
            Update,
            building_tooltip_system.run_if(in_state(GameState::Playing)),
        );
    }
}
