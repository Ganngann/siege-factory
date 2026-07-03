use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum GameState {
    #[default]
    Menu,
    Loading,
    Playing,
    GameOver,
}

/// True when entering Playing from a fresh game (not from load).
/// Setup systems check this via `run_if(is_fresh_game)`.
#[derive(Resource, Default)]
pub struct IsFreshGame(pub bool);
