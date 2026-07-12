pub mod archive;
pub mod belt;
pub mod build_bar;
pub mod building;
pub mod capsule;
pub mod compactor;
pub mod capsule_status;
pub mod components;
pub mod data_pad;
pub mod discovery;
pub mod discovery_components;
pub mod fluid;
pub mod game_components;
pub mod ground_items;
pub mod inspect;
pub mod menu;
pub mod placement;
pub mod player;
pub mod power;
pub mod power_components;
pub mod production;
pub mod recipe;
pub mod resource;
pub mod setup;
pub mod spatial;
pub mod tiered_structure;
pub mod tool;
pub mod ui;
pub mod ui_components;
pub mod unit_config;
pub mod window;

use crate::core::game_state::GameState;
use crate::core::schedule::GameplayStep;
use crate::core::toast::{ToastQueue, dismiss_persistent_toasts, toast_system};
use crate::core::tooltip::{TooltipText, tooltip_ui};
use crate::core::utils::silent_despawn;
use crate::ui::components::hud_text::{spawn_hud, despawn_hud};
use crate::ui::components::hand_crafting_list::populate_hand_crafting_list;
use crate::ui::components::hand_crafting_progress::update_hand_crafting_progress;
use crate::ui::global_panels::{toggle_crafting, toggle_inventory, cleanup_crafting, cleanup_inventory};
use bevy::ecs::hierarchy::ChildOf;
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;
use building::DefaultSettings;
use components::{Building, PeacefulMode, UiIsBlocking};
use menu::{MenuItems, MenuState};
use resource::ResourceRegistry;
use spatial::SpatialRegistry;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct PlayingSystems;

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
                if let Ok(p) = pickable_q.get(entity)
                    && p.should_block_lower {
                        return true;
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
        let mods = app.world().resource::<crate::core::modding::ModRegistry>().clone();

        app.insert_resource(ResourceRegistry::load(&mods));
        app.insert_resource(DefaultSettings::load(&mods));
        app.insert_resource(recipe::RecipeRegistry::load(&mods));
        let discovery_registry = discovery::DiscoveryRegistry::load(&mods);
        app.insert_resource(discovery::GlobalArchive::new(
            &discovery_registry.starter_recipes,
        ));
        app.insert_resource(discovery_registry);

        app.insert_resource(tool::ToolRegistry::load(&mods));
        app.insert_resource(crate::player::objective::ObjectiveRegistry::load(&mods));
        app.insert_resource(capsule_status::CapsuleStatusRegistry::load(&mods));
        app.init_resource::<crate::player::objective::ObjectiveState>();

        // Load registries + derive MenuDef in dependency order (avoids double-load)
        let building_registry = building::BuildingRegistry::load(&mods);
        let unit_cfg = unit_config::UnitConfig::load(&mods);
        let menu_def = menu::MenuDef::load(&mods, &building_registry, &unit_cfg);
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

        app.init_resource::<window::WindowDrag>();
        app.init_resource::<PeacefulMode>();
        app.init_resource::<MenuState>();
        app.init_resource::<MenuItems>();
        app.init_resource::<ToastQueue>();
        app.init_resource::<TooltipText>();
        app.init_resource::<UiIsBlocking>();
        app.init_resource::<player::PlayerWorldPos>();
        app.init_resource::<player::MiningTimer>();
        app.init_resource::<crate::player::crafting::CraftingProgress>();
        app.init_resource::<components::DragState>();
        app.init_resource::<crate::core::tutorial::TutorialState>();
        app.init_resource::<crate::core::tutorial::TutorialConditions>();
        app.init_resource::<crate::core::tutorial::TutorialHighlightEntity>();
        app.init_resource::<capsule::CapsuleConfig>();
        app.insert_resource(tiered_structure::ProgressionLogRegistry::default());
        app.init_resource::<tiered_structure::FinalCountdown>();
        app.insert_resource(Time::<Fixed>::from_hz(20.0));
        app.configure_sets(Update, PlayingSystems.run_if(in_state(GameState::Playing)));
        app.add_observer(placement::on_belt_drag_completed);
        app.add_observer(placement::on_deconstruct_area);
        app.add_observer(ground_items::spawn_ground_item_visual);
        app.add_systems(
            PreUpdate,
            update_ui_blocking.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            OnEnter(GameState::Playing),
            (
                player::setup_player.run_if(crate::save_load::is_fresh_game),
                capsule::spawn_capsule.run_if(crate::save_load::is_fresh_game),
                build_bar::spawn_menu_bar,
                crate::player::objective::spawn_objective_hud,
                spawn_hud,
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
                crate::player::objective::despawn_objective_hud,
                despawn_hud,
                cleanup_inventory,
                cleanup_crafting,
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
        app.add_systems(Update, placement::build_mode_input.in_set(PlayingSystems));
        app.add_systems(Update, placement::track_belt_drag.in_set(PlayingSystems));
        app.add_systems(Update, placement::handle_build_click.in_set(PlayingSystems));
        app.add_systems(
            Update,
            placement::handle_deconstruct_click_v2.in_set(PlayingSystems),
        );
        app.add_systems(
            Update,
            placement::track_deconstruct_drag.in_set(PlayingSystems),
        );
        app.add_systems(
            Update,
            placement::update_build_preview.in_set(PlayingSystems),
        );
        app.add_systems(
            Update,
            placement::deconstruct_drag_preview.in_set(PlayingSystems),
        );
        app.add_systems(Update, player::player_movement
            .in_set(PlayingSystems)
            .in_set(GameplayStep::PlayerInput));
        app.add_systems(Update, player::camera_follow_player
            .in_set(PlayingSystems)
            .in_set(GameplayStep::CameraFollow));
        app.add_systems(Update, player::builder_work.in_set(PlayingSystems));
        app.add_systems(Update, player::finish_construction.in_set(PlayingSystems));

        app.add_systems(Update, player::player_pickup_belt.in_set(PlayingSystems));
        app.add_systems(Update, capsule::update_capsule_visual.in_set(PlayingSystems));
        app.add_systems(Update, ground_items::player_pickup_ground_items.in_set(PlayingSystems));
        app.add_systems(Update, data_pad::interact_data_pad.in_set(PlayingSystems));
        app.add_systems(
            Update,
            crate::player::interact::contextual_interact.in_set(PlayingSystems),
        );
        app.add_systems(
            Update,
            tiered_structure::final_countdown_tick.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            crate::player::objective::advance_objectives.in_set(PlayingSystems),
        );
        app.add_systems(
            Update,
            crate::player::objective::update_objective_hud.in_set(PlayingSystems),
        );
        app.add_systems(
            Update,
            toggle_crafting.in_set(PlayingSystems),
        );
        app.add_systems(
            Update,
            crate::player::crafting::craft_button_system.in_set(PlayingSystems),
        );
        app.add_systems(
            Update,
            crate::player::crafting::crafting_tick.in_set(PlayingSystems),
        );
        app.add_systems(
            Update,
            populate_hand_crafting_list.in_set(PlayingSystems),
        );
        app.add_systems(
            Update,
            update_hand_crafting_progress.in_set(PlayingSystems),
        );
        app.add_systems(
            FixedUpdate,
            belt::advance_belt_slots.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            production::assembler_tick.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            power::burner_generator_tick.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            power::recipe_generator_tick.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            compactor::compactor_tick.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            fluid::fluid_pipe_transfer.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            fluid::water_pump_tick.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update, discovery::check_discoveries.in_set(PlayingSystems));
        app.add_systems(
            Update,
            archive::archive_delivery_check.in_set(PlayingSystems),
        );
        app.add_observer(discovery::on_discovery);
        app.add_systems(
            FixedUpdate,
            belt::building_output_tick.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update, toggle_inventory.in_set(PlayingSystems));
        app.add_systems(Update, ui::update_inventory_grids.in_set(PlayingSystems));
        app.add_systems(Update, ui::drag_start.in_set(PlayingSystems));
        app.add_systems(Update, ui::drag_update.in_set(PlayingSystems));
        app.add_systems(Update, ui::drag_end.in_set(PlayingSystems));
        app.add_systems(Update, window::close_window_system.in_set(PlayingSystems));
        app.add_systems(Update, build_bar::menu_navigation.in_set(PlayingSystems));
        app.add_systems(
            Update,
            build_bar::menu_bar_interaction.in_set(PlayingSystems),
        );
        app.add_systems(Update, build_bar::refresh_menu_bar.in_set(PlayingSystems));
        app.add_systems(Update, build_bar::update_menu_bar.in_set(PlayingSystems));
        app.add_systems(Update, inspect::cache_capsule_ui_data.in_set(PlayingSystems));
        app.add_systems(
            Update,
            inspect::building_inspect_click
                .in_set(PlayingSystems)
                .run_if(inspect::not_build_mode),
        );
        app.add_systems(Update, inspect::overlay_click_system.in_set(PlayingSystems));
        app.add_systems(Update, inspect::close_button_system.in_set(PlayingSystems));
        app.add_systems(
            Update,
            inspect::close_popup_on_escape.in_set(PlayingSystems),
        );
        app.add_systems(Update, inspect::active_toggle_system.in_set(PlayingSystems));
        app.add_systems(Update, crate::ui::components::recipe_row::recipe_row_click_system.in_set(PlayingSystems));
        app.add_systems(Update, crate::ui::components::recipe_category::populate_recipe_categories.in_set(PlayingSystems));
        app.add_systems(Update, crate::ui::components::data_list::populate_data_list.in_set(PlayingSystems));
        app.add_systems(Update, crate::ui::components::data_list::data_list_click_system.in_set(PlayingSystems));
        app.add_systems(Update, crate::ui::components::data_text::update_data_text_system.in_set(PlayingSystems));
        app.add_systems(Update, crate::ui::components::animate::animation_tick_system.in_set(PlayingSystems));
        app.add_systems(Update, crate::ui::components::key_value::update_capsule_statuses_system.in_set(PlayingSystems));
        app.add_systems(Update, crate::ui::components::wireframe::update_capsule_wireframe_system.in_set(PlayingSystems));
        app.add_systems(Update, inspect::farm_recruit_system.in_set(PlayingSystems));
        app.add_systems(
            Update,
            inspect::farm_crop_select_system.in_set(PlayingSystems),
        );
        app.add_systems(
            Update,
            inspect::sorter_resource_click_system.in_set(PlayingSystems),
        );
        app.add_systems(
            Update,
            inspect::sorter_invert_click_system.in_set(PlayingSystems),
        );
        app.add_systems(
            Update,
            inspect::upgrade_button_system.in_set(PlayingSystems),
        );
        app.add_systems(Update, window::drag_window_system.in_set(PlayingSystems));
        app.add_systems(Update, toast_system.in_set(PlayingSystems));
        app.add_systems(
            Update,
            dismiss_persistent_toasts.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(Update, tooltip_ui.in_set(PlayingSystems));
        app.add_systems(
            Update,
            (
                crate::core::tutorial::track_player_movement,
                crate::core::tutorial::track_item_collected,
                crate::core::tutorial::track_item_crafted,
                crate::core::tutorial::track_building_placed,
                crate::core::tutorial::tutorial_tick,
                crate::core::tutorial::tutorial_highlight_system,
            )
                .run_if(in_state(GameState::Playing))
                .in_set(PlayingSystems),
        );

        app.add_systems(
            Startup,
            |registry: Res<crate::core::modding::ModRegistry>,
             mut tutorial: ResMut<crate::core::tutorial::TutorialState>| {
                *tutorial = crate::core::tutorial::TutorialState::load(&registry);
            },
        );
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
    camera: Query<Entity, With<Camera2d>>,
) {
    for entity in &camera {
        silent_despawn(&mut commands, entity);
    }
}
