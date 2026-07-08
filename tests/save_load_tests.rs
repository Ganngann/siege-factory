use bevy::prelude::*;
use siege_factory::core::game_state::{GameState, IsFreshGame};
use siege_factory::core::input::KeyBindings;
use siege_factory::core::toast::ToastQueue;
use siege_factory::core::tutorial::TutorialState;
use siege_factory::enemy::components::{LastWave, WaveState};
use siege_factory::map::tile_grid::ChunkGrid;
use siege_factory::save_load::*;

fn test_dist() -> Vec<(String, u32)> {
    vec![("iron_ore".to_string(), 50)]
}

fn save_load_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.insert_resource(SaveManager::default());
    app.insert_resource(ShowPauseMenu(false));
    app.insert_resource(SaveRequested(false));
    app.insert_resource(LoadBuffer::default());
    app.insert_resource(IsFreshGame(true));
    app.insert_resource(ToastQueue::default());
    app.insert_resource(ChunkGrid::new(42, 50, 150, 35, 2, 5, test_dist()));
    app.insert_resource(WaveState::new(30.0));
    app.insert_resource(LastWave(0));
    app.insert_resource(TutorialState::default());
    app
}

// ════════════════════════════════════════════════════════════════
// toggle_pause_menu
// ════════════════════════════════════════════════════════════════

#[test]
fn toggle_pause_menu_esc_toggles_show() {
    let mut app = save_load_test_app();
    app.insert_resource(KeyBindings::load());
    app.add_systems(Update, toggle_pause_menu);

    assert!(!app.world().resource::<ShowPauseMenu>().0);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    app.update();

    assert!(app.world().resource::<ShowPauseMenu>().0);
}

#[test]
fn toggle_pause_menu_esc_toggles_back_off() {
    let mut app = save_load_test_app();
    app.insert_resource(KeyBindings::load());
    app.add_systems(Update, toggle_pause_menu);

    // Toggle on
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    app.update();
    assert!(app.world().resource::<ShowPauseMenu>().0);

    // Toggle off
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    app.update();
    assert!(!app.world().resource::<ShowPauseMenu>().0);
}

#[test]
fn toggle_pause_menu_other_key_no_toggle() {
    let mut app = save_load_test_app();
    app.insert_resource(KeyBindings::load());
    app.add_systems(Update, toggle_pause_menu);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyA);
    app.update();

    assert!(!app.world().resource::<ShowPauseMenu>().0);
}

#[test]
fn toggle_pause_menu_mouse_right_no_toggle() {
    let mut app = save_load_test_app();
    app.insert_resource(KeyBindings::load());
    app.add_systems(Update, toggle_pause_menu);

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    app.update();

    assert!(!app.world().resource::<ShowPauseMenu>().0);
}

// ════════════════════════════════════════════════════════════════
// spawn_pause_menu
// ════════════════════════════════════════════════════════════════

#[test]
fn spawn_pause_menu_creates_root_when_shown() {
    let mut app = save_load_test_app();
    app.add_systems(Update, spawn_pause_menu);

    app.world_mut().resource_mut::<ShowPauseMenu>().0 = true;
    app.update();

    let count = app
        .world_mut()
        .query::<&PauseMenuRoot>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1);
}

#[test]
fn spawn_pause_menu_no_creation_when_hidden() {
    let mut app = save_load_test_app();
    app.add_systems(Update, spawn_pause_menu);

    app.world_mut().resource_mut::<ShowPauseMenu>().0 = false;
    app.update();

    let count = app
        .world_mut()
        .query::<&PauseMenuRoot>()
        .iter(app.world())
        .count();
    assert_eq!(count, 0);
}

#[test]
fn spawn_pause_menu_despawns_when_hidden() {
    let mut app = save_load_test_app();
    app.add_systems(Update, spawn_pause_menu);

    // Show the menu
    app.world_mut().resource_mut::<ShowPauseMenu>().0 = true;
    app.update();
    assert_eq!(
        app.world_mut()
            .query::<&PauseMenuRoot>()
            .iter(app.world())
            .count(),
        1
    );

    // Hide the menu
    app.world_mut().resource_mut::<ShowPauseMenu>().0 = false;
    app.update();
    assert_eq!(
        app.world_mut()
            .query::<&PauseMenuRoot>()
            .iter(app.world())
            .count(),
        0
    );
}

#[test]
fn spawn_pause_menu_has_save_button() {
    let mut app = save_load_test_app();
    app.add_systems(Update, spawn_pause_menu);

    app.world_mut().resource_mut::<ShowPauseMenu>().0 = true;
    app.update();

    let count = app
        .world_mut()
        .query::<&SaveButton>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1);
}

#[test]
fn spawn_pause_menu_has_load_button() {
    let mut app = save_load_test_app();
    app.add_systems(Update, spawn_pause_menu);

    app.world_mut().resource_mut::<ShowPauseMenu>().0 = true;
    app.update();

    let count = app
        .world_mut()
        .query::<&LoadButton>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1);
}

#[test]
fn spawn_pause_menu_has_resume_button() {
    let mut app = save_load_test_app();
    app.add_systems(Update, spawn_pause_menu);

    app.world_mut().resource_mut::<ShowPauseMenu>().0 = true;
    app.update();

    let count = app
        .world_mut()
        .query::<&ResumeButton>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1);
}

#[test]
fn spawn_pause_menu_has_quit_button() {
    let mut app = save_load_test_app();
    app.add_systems(Update, spawn_pause_menu);

    app.world_mut().resource_mut::<ShowPauseMenu>().0 = true;
    app.update();

    let count = app
        .world_mut()
        .query::<&QuitButton>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1);
}

#[test]
fn spawn_pause_menu_does_not_duplicate() {
    let mut app = save_load_test_app();
    app.add_systems(Update, spawn_pause_menu);

    // Show twice
    app.world_mut().resource_mut::<ShowPauseMenu>().0 = true;
    app.update();
    app.update();

    let count = app
        .world_mut()
        .query::<&PauseMenuRoot>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1);
}

// ════════════════════════════════════════════════════════════════
// resume_interaction
// ════════════════════════════════════════════════════════════════

#[test]
fn resume_interaction_sets_show_false() {
    let mut app = save_load_test_app();
    app.add_systems(Update, (spawn_pause_menu, resume_interaction));

    // Show the menu first
    app.world_mut().resource_mut::<ShowPauseMenu>().0 = true;
    app.update();
    assert!(app.world().resource::<ShowPauseMenu>().0);

    // Simulate resume button press: find the entity with ResumeButton
    let resume_entity: Entity = {
        let mut q = app.world_mut().query_filtered::<Entity, With<ResumeButton>>();
        q.iter(app.world()).next().unwrap()
    };
    app.world_mut()
        .entity_mut(resume_entity)
        .insert(Interaction::Pressed);

    app.update();

    assert!(!app.world().resource::<ShowPauseMenu>().0);
}

// ════════════════════════════════════════════════════════════════
// save_interaction
// ════════════════════════════════════════════════════════════════

#[test]
fn save_interaction_sets_save_requested() {
    let mut app = save_load_test_app();
    app.add_systems(Update, (spawn_pause_menu, save_interaction));

    // Show the menu
    app.world_mut().resource_mut::<ShowPauseMenu>().0 = true;
    app.update();

    // Simulate save button press
    let save_entity: Entity = {
        let mut q = app.world_mut().query_filtered::<Entity, With<SaveButton>>();
        q.iter(app.world()).next().unwrap()
    };
    app.world_mut()
        .entity_mut(save_entity)
        .insert(Interaction::Pressed);

    app.update();

    assert!(app.world().resource::<SaveRequested>().0);
    assert!(!app.world().resource::<ShowPauseMenu>().0);
}

// ════════════════════════════════════════════════════════════════
// save_game (resource interaction)
// ════════════════════════════════════════════════════════════════

#[test]
fn save_game_no_crash_when_no_save_requested() {
    let mut app = save_load_test_app();
    app.add_systems(Update, save_game);

    app.world_mut()
        .spawn((Camera2d, Transform::default()));

    app.update();

    assert!(app.world().resource::<ToastQueue>().0.is_empty());
}

#[test]
fn save_game_triggers_on_save_requested() {
    let mut app = save_load_test_app();
    app.add_systems(Update, save_game);

    app.world_mut()
        .spawn((Camera2d, Transform::default()));

    app.world_mut().resource_mut::<SaveRequested>().0 = true;
    app.update();

    let queue = app.world().resource::<ToastQueue>();
    assert!(!queue.0.is_empty());
}

#[test]
fn save_game_clears_save_requested_flag() {
    let mut app = save_load_test_app();
    app.add_systems(Update, save_game);

    app.world_mut()
        .spawn((Camera2d, Transform::default()));

    app.world_mut().resource_mut::<SaveRequested>().0 = true;
    app.update();

    assert!(!app.world().resource::<SaveRequested>().0);
}

#[test]
fn save_game_triggers_on_f5() {
    let mut app = save_load_test_app();
    app.add_systems(Update, save_game);

    app.world_mut()
        .spawn((Camera2d, Transform::default()));

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::F5);
    app.update();

    let queue = app.world().resource::<ToastQueue>();
    assert!(!queue.0.is_empty());
}

// ════════════════════════════════════════════════════════════════
// cleanup_pause_menu
// ════════════════════════════════════════════════════════════════

#[test]
fn cleanup_pause_menu_despawns_all_roots() {
    let mut app = save_load_test_app();
    app.add_systems(Update, (spawn_pause_menu, cleanup_pause_menu));

    // Create a menu
    app.world_mut().resource_mut::<ShowPauseMenu>().0 = true;
    app.update();
    assert_eq!(
        app.world_mut()
            .query::<&PauseMenuRoot>()
            .iter(app.world())
            .count(),
        1
    );

    // Trigger cleanup by toggling hide + running cleanup
    app.world_mut().resource_mut::<ShowPauseMenu>().0 = false;
    app.update();

    // cleanup_pause_menu should have removed it
    assert_eq!(
        app.world_mut()
            .query::<&PauseMenuRoot>()
            .iter(app.world())
            .count(),
        0
    );
}

// ════════════════════════════════════════════════════════════════
// load_finalize (no data path)
// ════════════════════════════════════════════════════════════════

#[test]
fn load_finalize_no_data_goes_to_menu() {
    let mut app = save_load_test_app();
    app.init_resource::<siege_factory::economy::components::PeacefulMode>();
    app.init_resource::<IsFreshGame>();
    app.add_systems(Update, load_finalize);

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Loading);
    app.update();

    // No data in buffer
    app.update();

    assert_eq!(
        *app.world().resource::<State<GameState>>(),
        GameState::Menu
    );
}

// ════════════════════════════════════════════════════════════════
// is_fresh_game helper
// ════════════════════════════════════════════════════════════════

#[test]
fn is_fresh_game_returns_true_when_set() {
    let mut app = save_load_test_app();
    assert!(app.world().resource::<IsFreshGame>().0);
}

#[test]
fn is_fresh_game_returns_false_when_unset() {
    let mut app = save_load_test_app();
    app.world_mut().resource_mut::<IsFreshGame>().0 = false;
    assert!(!app.world().resource::<IsFreshGame>().0);
}

// ════════════════════════════════════════════════════════════════
// Resource defaults
// ════════════════════════════════════════════════════════════════

#[test]
fn save_manager_load_requested_default_is_none() {
    let mgr = SaveManager::default();
    assert!(mgr.load_requested.is_none());
}

#[test]
fn show_pause_menu_default_is_false() {
    let spm = ShowPauseMenu::default();
    assert!(!spm.0);
}

#[test]
fn save_requested_default_is_false() {
    let sr = SaveRequested::default();
    assert!(!sr.0);
}

#[test]
fn load_buffer_default_has_no_data() {
    let buf = LoadBuffer::default();
    assert!(buf.data.is_none());
}

#[test]
fn save_data_round_trip() {
    let data = SaveData {
        version: 1,
        game_seed: 42,
        camera: CameraSave {
            x: 1.0,
            y: 2.0,
            scale: 1.5,
        },
        wave: WaveSave {
            timer: 5.0,
            wave: 3,
            spawn_timer: 1.0,
            last_wave: 2,
        },
        chunk_deposits: std::collections::HashMap::new(),
        visited: std::collections::HashMap::new(),
        buildings: Vec::new(),
        enemies: Vec::new(),
        units: Vec::new(),
        tutorial: TutorialSave {
            current_index: 1,
            completed: false,
        },
    };

    let serialized = ron::to_string(&data).unwrap();
    let deserialized: SaveData = ron::from_str(&serialized).unwrap();
    assert_eq!(deserialized.version, 1);
    assert_eq!(deserialized.game_seed, 42);
    assert_eq!(deserialized.wave.wave, 3);
}

#[test]
fn tutorial_save_default_values() {
    let ts = TutorialSave::default();
    assert_eq!(ts.current_index, 0);
    assert!(!ts.completed);
}
