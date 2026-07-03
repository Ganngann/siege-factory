use bevy::prelude::*;
use crate::economy::components::Direction;
use crate::economy::resource::ResourceId;
use crate::map::components::TilePosition;

#[derive(Event)]
pub struct DeconstructAreaEvent {
    pub start: TilePosition,
    pub end: TilePosition,
}

#[derive(Event)]
pub struct DespawnDeposit(pub Entity);

#[derive(Event)]
pub struct DespawnEnemy(pub Entity);

#[derive(Event)]
pub struct SpawnBeltItemEvent {
    pub source_tile: TilePosition,
    pub resource: ResourceId,
}

#[derive(Event)]
pub struct BuildOrderEvent {
    pub kind: String,
    pub pos: TilePosition,
}

#[derive(Event)]
pub struct ToastEvent(pub String);

/// Emitted when a belt/splitter/sorter drag is completed.
/// The observer handles cost deduction + entity spawn/update.
#[derive(Event)]
pub struct BeltDragCompleted {
    pub kind: String,
    pub new_tiles: Vec<(i32, i32, Direction)>,
    pub existing: Vec<(i32, i32, Direction)>,
}

pub struct CleanupPlugin;

impl Plugin for CleanupPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(cleanup_deposits);
        app.add_observer(cleanup_enemies);
    }
}

fn cleanup_deposits(on: On<DespawnDeposit>, mut commands: Commands) {
    commands.entity(on.event().0).despawn();
}

fn cleanup_enemies(on: On<DespawnEnemy>, mut commands: Commands) {
    commands.entity(on.event().0).despawn();
}
