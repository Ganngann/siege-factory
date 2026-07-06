pub mod archive;
pub mod belt;
pub mod build_bar;
pub mod building;
pub mod components;
pub mod discovery;
pub mod inspect;
pub mod menu;
pub mod placement;
pub mod power;
pub mod production;
pub mod recipe;
pub mod resource;
pub mod setup;
pub mod spatial;
pub mod ui;
pub mod unit_config;

use crate::core::game_state::GameState;
use crate::core::toast::{ToastQueue, toast_system};
use crate::core::tooltip::{TooltipText, tooltip_ui};
use crate::core::utils::silent_despawn;
use bevy::ecs::hierarchy::ChildOf;
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;
use building::DefaultSettings;
use components::{Building, PeacefulMode, UiIsBlocking};
use menu::{MenuItems, MenuState};
use resource::ResourceRegistry;
use spatial::SpatialRegistry;
use ui::ResourceCountText;

pub fn update_ui_blocking(
    mut blocking: ResMut<UiIsBlocking>,
    hover_map: Res<HoverMap>,
    pickable_q: Query<&Pickable>,
    parent_q: Query<&ChildOf>,
) {
    blocking.0 = hover_map.iter().any(|(_, list)| {
        list.iter().any(|entry| {
            let mut entity = *entry.0;
            loop {
                if let Ok(p) = pickable_q.get(entity) {
                    if p.should_block_lower {
                        return true;
                    }
                }
                match parent_q.get(entity) {
                    Ok(child_of) => entity = child_of.0,
                    Err(_) => return false,
                }
            }
        })
    });
}

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ResourceRegistry::load());
        app.insert_resource(DefaultSettings::load());
        app.insert_resource(recipe::RecipeRegistry::load());
        app.insert_resource(discovery::DiscoveryRegistry::load());
        app.insert_resource(discovery::GlobalArchive::default());

        // Load registries + derive MenuDef in dependency order (avoids double-load)
        let building_registry = building::BuildingRegistry::load();
        let unit_cfg = unit_config::UnitConfig::load();
        let menu_def = menu::MenuDef::load(&building_registry, &unit_cfg);
        app.insert_resource(building_registry);
        app.insert_resource(unit_cfg);
        app.insert_resource(menu_def);

        app.init_resource::<SpatialRegistry>();
        app.init_resource::<power::PowerGrid>();
        app.init_resource::<components::BuildMode>();
        app.init_resource::<components::BeltDirection>();
        app.init_resource::<components::BuildPreview>();
        app.init_resource::<components::BeltDrag>();
        app.init_resource::<components::DeconstructMode>();
        app.init_resource::<components::DeconstructDrag>();
        app.init_resource::<components::BuildingPanel>();
        app.init_resource::<inspect::PanelDrag>();
        app.init_resource::<PeacefulMode>();
        app.init_resource::<MenuState>();
        app.init_resource::<MenuItems>();
        app.init_resource::<ToastQueue>();
        app.init_resource::<TooltipText>();
        app.init_resource::<UiIsBlocking>();
        app.insert_resource(Time::<Fixed>::from_hz(20.0));
        app.add_observer(placement::on_belt_drag_completed);
        app.add_observer(placement::on_deconstruct_area);
        app.add_systems(
            PreUpdate,
            update_ui_blocking.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            OnEnter(GameState::Playing),
            (
                setup::setup_hq.run_if(crate::save_load::is_fresh_game),
                build_bar::spawn_menu_bar,
            ),
        );
        app.add_systems(
            OnExit(GameState::Playing),
            (
                cleanup_playing_ui,
                cleanup_ghost,
                cleanup_buildings,
                build_bar::cleanup_menu_bar,
                inspect::cleanup_popup,
            ),
        );
        app.add_systems(
            PreUpdate,
            spatial::sync_spatial_registry.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            PreUpdate,
            power::detect_power_changes.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            PreUpdate,
            power::rebuild_power_grid.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            placement::build_mode_input.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            placement::track_belt_drag.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            placement::handle_build_click.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            placement::handle_deconstruct_click_v2.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            placement::track_deconstruct_drag.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            placement::update_build_preview.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            placement::deconstruct_drag_preview.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            belt::advance_belt_slots.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            production::assembler_tick.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            discovery::check_discoveries.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            archive::archive_delivery_check.run_if(in_state(GameState::Playing)),
        );
        app.add_observer(discovery::on_discovery);
        app.add_systems(
            FixedUpdate,
            belt::building_output_tick.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            ui::resource_count_ui.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            build_bar::menu_navigation.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            build_bar::menu_bar_interaction.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            build_bar::refresh_menu_bar.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            build_bar::update_menu_bar.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            inspect::building_inspect_click.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            inspect::overlay_click_system.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            inspect::close_button_system.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            inspect::close_popup_on_escape.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            inspect::active_toggle_system.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            inspect::recipe_change_system.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            inspect::recipe_selector_click.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            inspect::farm_recruit_system.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            inspect::farm_crop_select_system.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            inspect::sorter_resource_click_system.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            inspect::sorter_invert_click_system.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            inspect::drag_panel_system.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            (
                inspect::update_panel_header,
                inspect::update_panel_production,
                inspect::update_panel_inventory,
                inspect::update_panel_connections,
                inspect::update_panel_stats,
                inspect::update_panel_hp,
                inspect::update_panel_alerts,
                inspect::update_panel_power,
                inspect::update_farm_crop_text,
                inspect::update_farm_cultivator_count,
            )
                .run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update, toast_system.run_if(in_state(GameState::Playing)));
        app.add_systems(Update, tooltip_ui.run_if(in_state(GameState::Playing)));
    }
}

fn cleanup_ghost(mut commands: Commands, query: Query<Entity, With<components::Ghost>>) {
    for entity in &query {
        silent_despawn(&mut commands, entity);
    }
}

fn cleanup_buildings(
    mut commands: Commands,
    query: Query<Entity, With<Building>>,
    belt_query: Query<&crate::economy::belt::BeltSlots>,
) {
    for entity in &query {
        if let Ok(belt) = belt_query.get(entity) {
            for sprite_entity in belt.slot_sprites.iter().flatten() {
                silent_despawn(&mut commands, *sprite_entity);
            }
        }
        silent_despawn(&mut commands, entity);
    }
}

fn cleanup_playing_ui(
    mut commands: Commands,
    query: Query<Entity, With<ResourceCountText>>,
    camera: Query<Entity, With<Camera2d>>,
    waves: Query<Entity, With<crate::enemy::components::WaveCounterText>>,
    fps: Query<Entity, With<crate::rendering::FpsOverlay>>,
) {
    for entity in &query {
        silent_despawn(&mut commands, entity);
    }
    for entity in &camera {
        silent_despawn(&mut commands, entity);
    }
    for entity in &waves {
        silent_despawn(&mut commands, entity);
    }
    for entity in &fps {
        silent_despawn(&mut commands, entity);
    }
}
