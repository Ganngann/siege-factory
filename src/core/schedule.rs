use bevy::prelude::*;
use crate::core::game_state::GameState;
use crate::economy::components::BuildMode;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();
        app.add_systems(OnEnter(GameState::Loading), spawn_loading_ui);
        app.add_systems(OnExit(GameState::Loading), despawn_loading_ui);
        app.add_systems(Update, game_state_transition);
    }
}

#[derive(Component)]
struct LoadingUi;

fn spawn_loading_ui(mut commands: Commands) {
    commands.spawn((Camera2d, LoadingUi));
    commands
        .spawn((LoadingUi, Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        }))
        .with_children(|parent| {
            parent.spawn((
                LoadingUi,
                Text::new("SIEGE FACTORY"),
                TextFont::from_font_size(48.0),
                TextColor(Color::srgb(0.8, 0.8, 1.0)),
            ));
            parent.spawn((
                LoadingUi,
                Text::new("Build defenses  |  Survive waves  |  Automate everything"),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.6, 0.6, 0.8)),
            ));
            parent.spawn((LoadingUi, Text::new(""), TextFont::default(), TextColor(Color::WHITE)));
            parent.spawn((
                LoadingUi,
                Text::new("Press SPACE to start"),
                TextFont::from_font_size(20.0),
                TextColor(Color::WHITE),
            ));
        });
}

fn despawn_loading_ui(mut commands: Commands, query: Query<Entity, With<LoadingUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn game_state_transition(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut build_mode: Option<ResMut<BuildMode>>,
) {
    let mode_active = build_mode
        .as_ref()
        .map(|m| m.0.is_some())
        .unwrap_or(false);

    match state.get() {
        GameState::Loading => {
            if keys.just_pressed(KeyCode::Space) {
                next_state.set(GameState::Playing);
            }
        }
        GameState::Playing => {
            if keys.just_pressed(KeyCode::Escape) {
                if mode_active {
                    if let Some(ref mut bm) = build_mode {
                        bm.0 = None;
                    }
                } else {
                    next_state.set(GameState::GameOver);
                }
            }
        }
        GameState::GameOver => {
            if keys.just_pressed(KeyCode::KeyR) {
                next_state.set(GameState::Playing);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<BuildMode>();
        app.add_systems(Update, game_state_transition);
        app
    }

    #[test]
    fn initial_state_is_loading() {
        let mut app = test_app();
        app.update();
        assert_eq!(**app.world().resource::<State<GameState>>(), GameState::Loading);
    }

    #[test]
    fn no_keypress_no_transition() {
        let mut app = test_app();
        app.update();

        app.update();
        assert_eq!(**app.world().resource::<State<GameState>>(), GameState::Loading);
    }

    #[test]
    fn escape_cancels_build_mode_before_forfeit() {
        let mut app = test_app();
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        app.update();

        app.world_mut().resource_mut::<BuildMode>().0 = Some("wall".to_string());
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
        app.update();
        // Build mode cancelled, still Playing
        assert_eq!(**app.world().resource::<State<GameState>>(), GameState::Playing);
        assert!(app.world().resource::<BuildMode>().0.is_none());
    }

    #[test]
    fn nextstate_set_triggers_transition() {
        let mut app = test_app();
        app.update();
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        app.update();
        assert_eq!(**app.world().resource::<State<GameState>>(), GameState::Playing);
    }
}
