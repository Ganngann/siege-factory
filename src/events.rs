use bevy::prelude::*;
use crate::economy::resource::ResourceId;
use crate::map::components::TilePosition;

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
