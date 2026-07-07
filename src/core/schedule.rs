use crate::core::game_state::GameState;
use crate::core::input::KeyBindings;
use crate::core::main_menu::{self, MainMenuDef, MenuNav, RebindState};
use crate::core::settings::Settings;
use crate::economy::components::BuildMode;
use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings::load());
        let mut bindings = KeyBindings::load();
        {
            let settings = app.world().resource::<Settings>();
            bindings.apply_overrides(&settings.keybindings);
        }
        app.insert_resource(bindings);
        app.insert_resource(MainMenuDef::load());
        app.insert_resource(MenuNav::default());
        app.insert_resource(RebindState::default());
        app.init_state::<GameState>();
        app.add_systems(OnExit(GameState::Menu), main_menu::despawn_menu_ui);
        app.add_systems(
            Update,
            (
                game_state_transition,
                main_menu::menu_navigation,
                main_menu::menu_rebind_handler,
            )
                .run_if(in_state(GameState::Menu)),
        );
        app.add_systems(OnEnter(GameState::Playing), set_continuous_winit);
        app.add_systems(OnExit(GameState::Playing), set_reactive_winit);
    }
}

fn game_state_transition(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
    mut build_mode: Option<ResMut<BuildMode>>,
) {
    let mode_active = build_mode.as_ref().map(|m| m.0.is_some()).unwrap_or(false);

    match state.get() {
        GameState::Menu => {
            // Menu → Playing is handled by menu_navigation via StartGame action
        }
        GameState::Playing => {
            if bindings.just_pressed("cancel", &keys, &mouse) {
                if mode_active {
                    if let Some(ref mut bm) = build_mode {
                        bm.0 = None;
                    }
                } else {
                    next_state.set(GameState::GameOver);
                }
            }
        }
        GameState::Loading => {
            // Transitional state — no key handling needed
        }
        GameState::GameOver => {
            if bindings.just_pressed("restart", &keys, &mouse) {
                next_state.set(GameState::Playing);
            } else if bindings.just_pressed("cancel", &keys, &mouse) {
                next_state.set(GameState::Menu);
            }
        }
    }
}

fn set_continuous_winit(mut settings: ResMut<WinitSettings>) {
    settings.focused_mode = UpdateMode::Continuous;
}

fn set_reactive_winit(mut settings: ResMut<WinitSettings>) {
    *settings = WinitSettings::desktop_app();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.insert_resource(KeyBindings::load());
        app.insert_resource(Settings::load());
        app.insert_resource(MainMenuDef::load());
        app.insert_resource(MenuNav::default());
        app.insert_resource(RebindState::default());
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<ButtonInput<MouseButton>>();
        app.init_resource::<BuildMode>();
        app.add_systems(Update, game_state_transition);
        app
    }

    #[test]
    fn initial_state_is_menu() {
        let mut app = test_app();
        app.update();
        assert_eq!(
            **app.world().resource::<State<GameState>>(),
            GameState::Menu
        );
    }

    #[test]
    fn no_keypress_no_transition() {
        let mut app = test_app();
        app.update();

        app.update();
        assert_eq!(
            **app.world().resource::<State<GameState>>(),
            GameState::Menu
        );
    }

    #[test]
    fn escape_cancels_build_mode_before_forfeit() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        app.world_mut().resource_mut::<BuildMode>().0 = Some("wall".to_string());
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        // Build mode cancelled, still Playing
        assert_eq!(
            **app.world().resource::<State<GameState>>(),
            GameState::Playing
        );
        assert!(app.world().resource::<BuildMode>().0.is_none());
    }

    #[test]
    fn nextstate_set_triggers_transition() {
        let mut app = test_app();
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();
        assert_eq!(
            **app.world().resource::<State<GameState>>(),
            GameState::Playing
        );
    }
}
