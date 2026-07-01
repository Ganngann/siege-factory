pub mod building;
pub mod recipe;
pub mod resource;
pub mod systems;
pub mod ui;
pub mod unit_config;

use bevy::prelude::*;
use crate::core::game_state::GameState;
use building::BuildingRegistry;
use resource::ResourceRegistry;
use ui::{OreCountText, BuildModeText};

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ResourceRegistry::load());
        app.insert_resource(BuildingRegistry::load());
        app.insert_resource(recipe::RecipeRegistry::load());
        app.insert_resource(unit_config::UnitConfig::load());
        app.init_resource::<systems::BuildMode>();
        app.init_resource::<systems::BeltDirection>();
        app.init_resource::<systems::BuildPreview>();
        app.add_systems(OnEnter(GameState::Playing), (
            systems::setup_hq,
            systems::place_ore_deposits,
        ));
        app.add_systems(OnExit(GameState::Playing), (cleanup_playing_ui, cleanup_ghost));
        app.add_systems(Update, (
            systems::build_mode_input,
            systems::handle_build_click,
            systems::update_build_preview,
            systems::production_tick,
            systems::assembler_tick,
            systems::move_belt_items,
            ui::ore_count_ui,
            ui::build_mode_indicator,
        ).run_if(in_state(GameState::Playing)));
    }
}

fn cleanup_ghost(mut commands: Commands, query: Query<Entity, With<systems::Ghost>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::type_complexity)]
fn cleanup_playing_ui(mut commands: Commands, query: Query<Entity, Or<(With<OreCountText>, With<BuildModeText>)>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
