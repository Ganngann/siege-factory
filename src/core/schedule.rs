use crate::core::game_font::{self, GameFont};
use crate::core::game_state::{GameState, IsFreshGame};
use crate::core::input::KeyBindings;
use crate::core::main_menu::{self, MainMenuDef, MenuNav, RebindState};
use crate::core::settings::Settings;
use crate::economy::components::BuildMode;
use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameplayStep {
    PlayerInput,
    CameraFollow,
    ChunkManagement,
    FogOfWar,
}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings::load());
        let mods = app.world().resource::<crate::core::modding::ModRegistry>().clone();
        let mut bindings = KeyBindings::load(&mods);
        {
            let settings = app.world().resource::<Settings>();
            bindings.apply_overrides(&settings.keybindings);
        }
        app.insert_resource(bindings);
        app.insert_resource(MainMenuDef::load(&mods));
        app.insert_resource(MenuNav::default());
        app.insert_resource(RebindState::default());
        app.init_state::<GameState>();
        app.init_resource::<GameFont>();
        app.add_systems(Startup, game_font::load_game_font);
        app.add_systems(OnExit(GameState::Menu), main_menu::despawn_menu_ui);
        app.add_systems(
            Update,
            game_state_transition.run_if(not(in_state(GameState::Loading))),
        );
        app.add_systems(
            Update,
            main_menu::menu_navigation.run_if(in_state(GameState::Menu)),
        );
        app.add_systems(
            Update,
            main_menu::menu_rebind_handler.run_if(in_state(GameState::Menu)),
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
    mut fresh_game: Option<ResMut<IsFreshGame>>,
) {
    let mode_active = build_mode.as_ref().map(|m| m.0.is_some()).unwrap_or(false);

    match state.get() {
        GameState::Playing => {
            if mode_active && bindings.just_pressed("cancel", &keys, &mouse)
                && let Some(ref mut bm) = build_mode {
                    bm.0 = None;
                }
        }
        GameState::GameOver => {
            if bindings.just_pressed("restart", &keys, &mouse) {
                if let Some(ref mut fg) = fresh_game {
                    fg.0 = true;
                }
                next_state.set(GameState::Playing);
            } else if bindings.just_pressed("cancel", &keys, &mouse) {
                next_state.set(GameState::Menu);
            }
        }
        _ => {}
    }
}

fn set_continuous_winit(mut settings: ResMut<WinitSettings>) {
    settings.focused_mode = UpdateMode::Continuous;
}

fn set_reactive_winit(mut settings: ResMut<WinitSettings>) {
    *settings = WinitSettings::desktop_app();
}


