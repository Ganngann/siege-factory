use crate::economy::components::Direction;
use crate::map::components::TilePosition;
use bevy::prelude::*;

#[derive(Event)]
pub struct SpawnGroundItemEvent {
    pub resource_id: String,
    pub amount: u32,
    pub position: TilePosition,
}

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
pub struct BuildOrderEvent {
    pub kind: String,
    pub pos: TilePosition,
}

/// Emitted by logical combat systems; consumed by rendering to spawn projectile visuals.
#[derive(Event)]
pub struct SpawnProjectileEvent {
    pub target: Entity,
    pub speed: f32,
    pub damage: u32,
    pub origin: Vec3,
    pub color: Color,
}

/// Emitted when a belt/splitter/sorter drag is completed.
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
