use std::collections::HashMap;

use bevy::prelude::*;
use siege_factory::core::game_state::GameState;
use siege_factory::core::input::KeyBindings;
use siege_factory::core::settings::Settings;
use siege_factory::core::toast::{toast_system, ToastMessage, ToastQueue};
use siege_factory::core::tutorial::{
    tutorial_tick, TutorialConditions, TutorialHighlightEntity, TutorialState, TutorialStepDef,
};
use siege_factory::economy::components::Building;
use siege_factory::economy::player::PlayerWorldPos;
use siege_factory::map::components::TilePosition;
use siege_factory::map::config::MapConfig;
use siege_factory::rendering::config::VisualsConfig;
use siege_factory::core::modding::ModRegistry;


// ════════════════════════════════════════════════════════════════
// Helpers
// ════════════════════════════════════════════════════════════════

fn test_mods() -> ModRegistry { ModRegistry::for_test() }
fn tutorial_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<TutorialState>();
    app.init_resource::<TutorialConditions>();
    app.init_resource::<TutorialHighlightEntity>();
    app.init_resource::<ToastQueue>();
    app.init_resource::<PlayerWorldPos>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.insert_resource(MapConfig::load(&test_mods()));
    app.add_systems(Update, tutorial_tick);
    app
}

fn make_step(condition: &str, params: Vec<(&str, &str)>) -> TutorialStepDef {
    TutorialStepDef {
        id: "test".to_string(),
        toast: "toast msg".to_string(),
        condition: condition.to_string(),
        params: params
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        highlight: None,
    }
}

fn game_state_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app
}

// ════════════════════════════════════════════════════════════════
// Pure tests: TutorialConditions defaults
// ════════════════════════════════════════════════════════════════

#[test]
fn tutorial_conditions_default_is_zero() {
    let c = TutorialConditions::default();
    assert_eq!(c.player_moved_distance, 0.0);
    assert_eq!(c.items_collected, 0);
    assert!(c.items_crafted.is_empty());
    assert_eq!(c.buildings_placed, 0);
    assert!(c.building_types_placed.is_empty());
    assert!(!c.has_interacted_with_structure);
    assert!(c.structures_interacted.is_empty());
}

// ════════════════════════════════════════════════════════════════
// Settings serialization round-trip
// ════════════════════════════════════════════════════════════════

#[test]
fn settings_serialization_round_trip() {
    let mut keybindings = HashMap::new();
    keybindings.insert("cancel".to_string(), "Escape".to_string());
    keybindings.insert("place".to_string(), "MouseLeft".to_string());
    let settings = Settings {
        keybindings: keybindings.clone(),
    };

    let toml_str = toml::to_string(&settings).unwrap();
    let loaded: Settings = toml::from_str(&toml_str).unwrap();

    assert_eq!(settings.keybindings, loaded.keybindings);
}

#[test]
fn settings_default_has_empty_keybindings() {
    let settings = Settings::default();
    assert!(settings.keybindings.is_empty());
}

#[test]
fn settings_round_trip_preserves_all_entries() {
    let mut keybindings = HashMap::new();
    keybindings.insert("start_game".to_string(), "Space".to_string());
    keybindings.insert("cancel".to_string(), "Escape".to_string());
    keybindings.insert("restart".to_string(), "KeyR".to_string());
    keybindings.insert("place".to_string(), "MouseLeft".to_string());
    keybindings.insert("mine".to_string(), "KeyE".to_string());
    let settings = Settings {
        keybindings: keybindings.clone(),
    };

    let toml_str = toml::to_string(&settings).unwrap();
    let loaded: Settings = toml::from_str(&toml_str).unwrap();

    assert_eq!(settings.keybindings.len(), loaded.keybindings.len());
    for (k, v) in &settings.keybindings {
        assert_eq!(loaded.keybindings.get(k).unwrap(), v);
    }
}

// ════════════════════════════════════════════════════════════════
// KeyBindings loading and just_pressed
// ════════════════════════════════════════════════════════════════

#[test]
fn keybindings_loads_expected_actions() {
    let bindings = KeyBindings::load(&test_mods());
    // Actions defined in data/keybindings.toml
    let _ = bindings.get("cancel");
    let _ = bindings.get("restart");
    let _ = bindings.get("place");
    let _ = bindings.get("mine");
    let _ = bindings.get("pickup_belt");
}

#[test]
fn keybindings_just_pressed_keyboard() {
    let bindings = KeyBindings::load(&test_mods());
    let mut keys = ButtonInput::<KeyCode>::default();
    let mouse = ButtonInput::<MouseButton>::default();

    assert!(!bindings.just_pressed("cancel", &keys, &mouse));
    keys.press(KeyCode::Escape);
    assert!(bindings.just_pressed("cancel", &keys, &mouse));
}

#[test]
fn keybindings_just_pressed_mouse() {
    let bindings = KeyBindings::load(&test_mods());
    let keys = ButtonInput::<KeyCode>::default();
    let mut mouse = ButtonInput::<MouseButton>::default();

    assert!(!bindings.just_pressed("place", &keys, &mouse));
    mouse.press(MouseButton::Left);
    assert!(bindings.just_pressed("place", &keys, &mouse));
}

#[test]
fn keybindings_just_pressed_returns_false_for_unbound_key() {
    let bindings = KeyBindings::load(&test_mods());
    let mut keys = ButtonInput::<KeyCode>::default();
    let mouse = ButtonInput::<MouseButton>::default();

    keys.press(KeyCode::KeyZ);
    assert!(!bindings.just_pressed("cancel", &keys, &mouse));
}

// ════════════════════════════════════════════════════════════════
// evaluate_condition tested via tutorial_tick system
// ════════════════════════════════════════════════════════════════

// ── player_moved_distance ──

#[test]
fn tutorial_player_moved_distance_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("player_moved_distance", vec![("distance", "5.0")])];
    app.world_mut().resource_mut::<TutorialConditions>().player_moved_distance = 10.0;

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 1);
    assert!(!app.world().resource::<ToastQueue>().0.is_empty());
}

#[test]
fn tutorial_player_moved_distance_not_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("player_moved_distance", vec![("distance", "5.0")])];
    app.world_mut().resource_mut::<TutorialConditions>().player_moved_distance = 3.0;

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 0);
    assert!(app.world().resource::<ToastQueue>().0.is_empty());
}

#[test]
fn tutorial_player_moved_distance_default_threshold() {
    let mut app = tutorial_test_app();
    // No distance param → default is 5.0
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("player_moved_distance", vec![])];
    app.world_mut().resource_mut::<TutorialConditions>().player_moved_distance = 6.0;

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 1);
}

#[test]
fn tutorial_player_moved_distance_exact_threshold() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("player_moved_distance", vec![("distance", "5.0")])];
    app.world_mut().resource_mut::<TutorialConditions>().player_moved_distance = 5.0;

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 1);
}

// ── items_collected ──

#[test]
fn tutorial_items_collected_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("items_collected", vec![("count", "2")])];
    app.world_mut().resource_mut::<TutorialConditions>().items_collected = 5;

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 1);
}

#[test]
fn tutorial_items_collected_not_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("items_collected", vec![("count", "10")])];
    app.world_mut().resource_mut::<TutorialConditions>().items_collected = 3;

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 0);
}

#[test]
fn tutorial_items_collected_default_count() {
    let mut app = tutorial_test_app();
    // No count param → default is 1
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("items_collected", vec![])];
    app.world_mut().resource_mut::<TutorialConditions>().items_collected = 1;

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 1);
}

// ── item_crafted ──

#[test]
fn tutorial_item_crafted_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("item_crafted", vec![("recipe_id", "iron_plate")])];
    let mut crafted = HashMap::new();
    crafted.insert("iron_plate".to_string(), 1u32);
    app.world_mut().resource_mut::<TutorialConditions>().items_crafted = crafted;

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 1);
}

#[test]
fn tutorial_item_crafted_not_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("item_crafted", vec![("recipe_id", "iron_plate")])];
    // No items crafted

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 0);
}

#[test]
fn tutorial_item_crafted_wrong_recipe() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("item_crafted", vec![("recipe_id", "iron_plate")])];
    let mut crafted = HashMap::new();
    crafted.insert("copper_plate".to_string(), 1u32);
    app.world_mut().resource_mut::<TutorialConditions>().items_crafted = crafted;

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 0);
}

#[test]
fn tutorial_item_crafted_no_recipe_id_returns_false() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("item_crafted", vec![])];
    let mut crafted = HashMap::new();
    crafted.insert("iron_plate".to_string(), 1u32);
    app.world_mut().resource_mut::<TutorialConditions>().items_crafted = crafted;

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 0);
}

// ── building_placed ──

#[test]
fn tutorial_building_placed_with_id_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("building_placed", vec![("building_id", "miner")])];
    let mut types = HashMap::new();
    types.insert("miner".to_string(), 1u32);
    app.world_mut().resource_mut::<TutorialConditions>().building_types_placed = types;

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 1);
}

#[test]
fn tutorial_building_placed_with_id_wrong_type() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("building_placed", vec![("building_id", "miner")])];
    let mut types = HashMap::new();
    types.insert("wall".to_string(), 1u32);
    app.world_mut().resource_mut::<TutorialConditions>().building_types_placed = types;

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 0);
}

#[test]
fn tutorial_building_placed_without_id_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("building_placed", vec![])];
    app.world_mut().resource_mut::<TutorialConditions>().buildings_placed = 1;

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 1);
}

#[test]
fn tutorial_building_placed_without_id_not_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("building_placed", vec![])];
    // buildings_placed defaults to 0

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 0);
}

// ── player_near_structure ──

#[test]
fn tutorial_player_near_structure_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("player_near_structure", vec![("distance", "3.0")])];
    *app.world_mut().resource_mut::<PlayerWorldPos>() = PlayerWorldPos(Vec3::ZERO);

    app.world_mut().spawn((
        Building {
            kind: "miner".to_string(),
            name: "Miner".to_string(),
        },
        TilePosition { x: 1, y: 0 },
    ));

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 1);
}

#[test]
fn tutorial_player_near_structure_not_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("player_near_structure", vec![("distance", "1.0")])];
    *app.world_mut().resource_mut::<PlayerWorldPos>() = PlayerWorldPos(Vec3::ZERO);

    app.world_mut().spawn((
        Building {
            kind: "miner".to_string(),
            name: "Miner".to_string(),
        },
        TilePosition { x: 100, y: 100 },
    ));

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 0);
}

#[test]
fn tutorial_player_near_structure_filters_by_id() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps = vec![make_step(
        "player_near_structure",
        vec![("distance", "3.0"), ("structure_id", "wall")],
    )];
    *app.world_mut().resource_mut::<PlayerWorldPos>() = PlayerWorldPos(Vec3::ZERO);

    // Miner is nearby but structure_id expects "wall" → should NOT match
    app.world_mut().spawn((
        Building {
            kind: "miner".to_string(),
            name: "Miner".to_string(),
        },
        TilePosition { x: 1, y: 0 },
    ));

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 0);
}

#[test]
fn tutorial_player_near_structure_filters_by_id_match() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps = vec![make_step(
        "player_near_structure",
        vec![("distance", "3.0"), ("structure_id", "wall")],
    )];
    *app.world_mut().resource_mut::<PlayerWorldPos>() = PlayerWorldPos(Vec3::ZERO);

    // Wall is nearby and matches structure_id → should match
    app.world_mut().spawn((
        Building {
            kind: "wall".to_string(),
            name: "Wall".to_string(),
        },
        TilePosition { x: 1, y: 0 },
    ));

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 1);
}

// ── structure_interacted ──

#[test]
fn tutorial_structure_interacted_with_id_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("structure_interacted", vec![("structure_id", "miner")])];
    let mut interacted = HashMap::new();
    interacted.insert("miner".to_string(), true);
    app.world_mut().resource_mut::<TutorialConditions>().structures_interacted = interacted;

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 1);
}

#[test]
fn tutorial_structure_interacted_with_id_not_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("structure_interacted", vec![("structure_id", "miner")])];
    // No interactions recorded

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 0);
}

#[test]
fn tutorial_structure_interacted_without_id_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("structure_interacted", vec![])];
    app.world_mut().resource_mut::<TutorialConditions>().has_interacted_with_structure = true;

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 1);
}

#[test]
fn tutorial_structure_interacted_without_id_not_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("structure_interacted", vec![])];
    // has_interacted_with_structure defaults to false

    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 0);
}

// ── unknown condition ──

#[test]
fn tutorial_unknown_condition_returns_true() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("totally_unknown_condition", vec![])];

    app.update();

    // Unknown conditions return true (per the match arm with warn!)
    assert_eq!(app.world().resource::<TutorialState>().current_index, 1);
}

// ════════════════════════════════════════════════════════════════
// tutorial_tick: early-return paths
// ════════════════════════════════════════════════════════════════

#[test]
fn tutorial_tick_returns_early_when_completed() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("player_moved_distance", vec![("distance", "0.0")])];
    app.world_mut().resource_mut::<TutorialState>().completed = true;

    app.update();

    assert!(app.world().resource::<ToastQueue>().0.is_empty());
}

#[test]
fn tutorial_tick_returns_early_when_no_steps() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialConditions>().player_moved_distance = 999.0;

    app.update();

    assert!(app.world().resource::<ToastQueue>().0.is_empty());
}

// ════════════════════════════════════════════════════════════════
// Tab skipping
// ════════════════════════════════════════════════════════════════

#[test]
fn tutorial_tab_skips_step_when_condition_not_met() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps =
        vec![make_step("player_moved_distance", vec![("distance", "999.0")])];
    // Condition not met (0.0 < 999.0), but Tab should skip

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Tab);
    app.update();

    assert_eq!(app.world().resource::<TutorialState>().current_index, 1);
    assert!(!app.world().resource::<ToastQueue>().0.is_empty());
}

#[test]
fn tutorial_tab_skips_multiple_steps() {
    let mut app = tutorial_test_app();
    app.world_mut().resource_mut::<TutorialState>().steps = vec![
        make_step("player_moved_distance", vec![("distance", "999.0")]),
        make_step("items_collected", vec![("count", "999")]),
    ];

    // First Tab: skip step 0
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Tab);
    app.update();
    assert_eq!(app.world().resource::<TutorialState>().current_index, 1);

    // Second Tab: skip step 1 → completed
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Tab);
    app.update();
    assert_eq!(app.world().resource::<TutorialState>().current_index, 2);
    assert!(app.world().resource::<TutorialState>().completed);
}

// ════════════════════════════════════════════════════════════════
// ToastQueue + toast_system
// ════════════════════════════════════════════════════════════════

#[test]
fn toast_system_drains_queue_into_entities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ToastQueue>();
    app.insert_resource(VisualsConfig::load(&test_mods()));
    app.add_systems(Update, toast_system);

    app.world_mut()
        .resource_mut::<ToastQueue>()
        .0
        .push("Hello".to_string());
    app.world_mut()
        .resource_mut::<ToastQueue>()
        .0
        .push("World".to_string());

    app.update();

    assert!(app.world().resource::<ToastQueue>().0.is_empty());
    let mut q = app.world_mut().query::<&ToastMessage>();
    let count = q.iter(app.world()).count();
    assert_eq!(count, 2);
}

#[test]
fn toast_system_empty_queue_no_entities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ToastQueue>();
    app.insert_resource(VisualsConfig::load(&test_mods()));
    app.add_systems(Update, toast_system);

    app.update();

    let mut q = app.world_mut().query::<&ToastMessage>();
    let count = q.iter(app.world()).count();
    assert_eq!(count, 0);
}

// ════════════════════════════════════════════════════════════════
// GameState transitions
// ════════════════════════════════════════════════════════════════

#[test]
fn game_state_initial_is_menu() {
    let app = game_state_test_app();
    assert_eq!(*app.world().resource::<State<GameState>>(), GameState::Menu);
}

#[test]
fn game_state_menu_to_playing() {
    let mut app = game_state_test_app();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();
    assert_eq!(*app.world().resource::<State<GameState>>(), GameState::Playing);
}

#[test]
fn game_state_playing_to_game_over() {
    let mut app = game_state_test_app();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::GameOver);
    app.update();
    assert_eq!(
        *app.world().resource::<State<GameState>>(),
        GameState::GameOver
    );
}

#[test]
fn game_state_game_over_to_menu() {
    let mut app = game_state_test_app();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::GameOver);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Menu);
    app.update();
    assert_eq!(*app.world().resource::<State<GameState>>(), GameState::Menu);
}

#[test]
fn game_state_full_cycle_menu_playing_gameover_menu() {
    let mut app = game_state_test_app();

    assert_eq!(*app.world().resource::<State<GameState>>(), GameState::Menu);

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();
    assert_eq!(*app.world().resource::<State<GameState>>(), GameState::Playing);

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::GameOver);
    app.update();
    assert_eq!(
        *app.world().resource::<State<GameState>>(),
        GameState::GameOver
    );

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Menu);
    app.update();
    assert_eq!(*app.world().resource::<State<GameState>>(), GameState::Menu);
}
