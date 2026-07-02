pub mod belt;
pub mod build_bar;
pub mod building;
pub mod components;
pub mod inspect;
pub mod placement;
pub mod production;
pub mod recipe;
pub mod resource;
pub mod setup;
pub mod ui;
pub mod unit_config;

use bevy::prelude::*;
use crate::core::game_state::GameState;
use crate::core::toast::{toast_system, ToastQueue};
use crate::core::tooltip::{tooltip_ui, TooltipText};
use building::BuildingRegistry;
use building::DefaultSettings;
use resource::ResourceRegistry;
use ui::OreCountText;

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ResourceRegistry::load());
        app.insert_resource(BuildingRegistry::load());
        app.insert_resource(DefaultSettings::load());
        app.insert_resource(recipe::RecipeRegistry::load());
        app.insert_resource(unit_config::UnitConfig::load());
        app.init_resource::<components::BuildMode>();
        app.init_resource::<components::BeltDirection>();
        app.init_resource::<components::BuildPreview>();
        app.init_resource::<components::BeltDrag>();
        app.init_resource::<components::DeconstructMode>();
        app.init_resource::<components::BuildingPopup>();
        app.init_resource::<ToastQueue>();
        app.init_resource::<TooltipText>();
        app.add_observer(belt::belt_item_placer);
        app.add_observer(placement::on_belt_drag_completed);
        app.add_systems(OnEnter(GameState::Playing), (
            setup::setup_hq,
            setup::place_ore_deposits,
            build_bar::spawn_build_bar,
        ));
        app.add_systems(OnExit(GameState::Playing), (
            cleanup_playing_ui,
            cleanup_ghost,
            build_bar::cleanup_build_bar,
            inspect::cleanup_popup,
        ));
        app.add_systems(Update,
            placement::build_mode_input.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update,
            placement::track_belt_drag.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update,
            placement::handle_build_click.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update,
            placement::handle_deconstruct_click_v2.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update,
            placement::update_build_preview.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update,
            production::production_tick.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update,
            production::assembler_tick.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update,
            belt::advance_belt_slots.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update,
            belt::animate_belt_positions.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update,
            ui::ore_count_ui.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update,
            build_bar::build_bar_interaction.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update,
            build_bar::update_build_bar.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update,
            inspect::building_inspect_click.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update,
            inspect::sorter_toggle_click.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update,
            toast_system.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update,
            tooltip_ui.run_if(in_state(GameState::Playing)),
        );
    }
}

fn cleanup_ghost(mut commands: Commands, query: Query<Entity, With<components::Ghost>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn cleanup_playing_ui(mut commands: Commands, query: Query<Entity, With<OreCountText>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
