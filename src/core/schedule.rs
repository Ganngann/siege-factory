use bevy::prelude::*;
use crate::core::game_state::GameState;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();
        app.add_systems(Update, game_state_transition);
    }
}

fn compute_next_state(state: &GameState, keys: &ButtonInput<KeyCode>) -> Option<GameState> {
    match state {
        GameState::Loading if keys.just_pressed(KeyCode::Space) => Some(GameState::Playing),
        GameState::Playing if keys.just_pressed(KeyCode::Escape) => Some(GameState::GameOver),
        GameState::GameOver if keys.just_pressed(KeyCode::KeyR) => Some(GameState::Playing),
        _ => None,
    }
}

fn game_state_transition(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if let Some(next) = compute_next_state(&state, &keys) {
        next_state.set(next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_is_loading() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.update();
        assert_eq!(**app.world().resource::<State<GameState>>(), GameState::Loading);
    }

    #[test]
    fn compute_space_transitions_loading_to_playing() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::Space);
        assert_eq!(compute_next_state(&GameState::Loading, &keys), Some(GameState::Playing));
    }

    #[test]
    fn compute_escape_transitions_playing_to_gameover() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::Escape);
        assert_eq!(compute_next_state(&GameState::Playing, &keys), Some(GameState::GameOver));
    }

    #[test]
    fn compute_r_transitions_gameover_to_playing() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyR);
        assert_eq!(compute_next_state(&GameState::GameOver, &keys), Some(GameState::Playing));
    }

    #[test]
    fn compute_no_match_returns_none() {
        let keys = ButtonInput::<KeyCode>::default();
        assert_eq!(compute_next_state(&GameState::Loading, &keys), None);
        assert_eq!(compute_next_state(&GameState::Playing, &keys), None);
        assert_eq!(compute_next_state(&GameState::GameOver, &keys), None);
    }

    #[test]
    fn nextstate_set_triggers_transition() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.update();
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        app.update();
        assert_eq!(**app.world().resource::<State<GameState>>(), GameState::Playing);
    }
}
