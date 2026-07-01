use std::collections::HashSet;
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

pub struct CleanupPlugin;

impl Plugin for CleanupPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DespawnDeposit>();
        app.add_event::<DespawnEnemy>();
        app.add_event::<SpawnBeltItemEvent>();
        app.add_systems(Last, cleanup_deposits);
        app.add_systems(Last, cleanup_enemies);
    }
}

fn cleanup_deposits(
    mut commands: Commands,
    mut events: EventReader<DespawnDeposit>,
) {
    let mut seen = HashSet::new();
    for ev in events.read() {
        if seen.insert(ev.0) {
            commands.entity(ev.0).despawn();
        }
    }
}

fn cleanup_enemies(
    mut commands: Commands,
    mut events: EventReader<DespawnEnemy>,
) {
    let mut seen = HashSet::new();
    for ev in events.read() {
        if seen.insert(ev.0) {
            commands.entity(ev.0).despawn();
        }
    }
}
