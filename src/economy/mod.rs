pub mod building;
pub mod belt;
pub mod recipe;
pub mod resource;
pub mod systems;
pub mod ui;
pub mod unit_config;
pub mod build_bar;

use bevy::prelude::*;
use crate::core::game_state::GameState;
use building::BuildingRegistry;
use resource::ResourceRegistry;
use ui::OreCountText;

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
        app.add_event::<systems::SetBuildModeEvent>();
        app.add_systems(OnEnter(GameState::Playing), (
            systems::setup_hq,
            systems::place_ore_deposits,
            build_bar::spawn_build_bar,
        ));
        app.add_systems(OnExit(GameState::Playing), (
            cleanup_playing_ui,
            cleanup_ghost,
            build_bar::cleanup_build_bar,
        ));
        app.add_systems(Update, (
            systems::build_mode_input,
            systems::handle_build_click,
            systems::update_build_preview,
            systems::production_tick,
            belt::belt_item_placer,
            systems::assembler_tick,
            belt::advance_belt_slots,
            belt::animate_belt_positions,
            ui::ore_count_ui,
            build_bar::build_bar_interaction,
            build_bar::update_build_bar,
        ).run_if(in_state(GameState::Playing)));
    }
}

fn cleanup_ghost(mut commands: Commands, query: Query<Entity, With<systems::Ghost>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn cleanup_playing_ui(mut commands: Commands, query: Query<Entity, With<OreCountText>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
