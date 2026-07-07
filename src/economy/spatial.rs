use crate::economy::components::{Building, OccupiedTiles};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource)]
pub struct SpatialRegistry {
    pub(crate) map: HashMap<(i32, i32), Entity>,
    pub(crate) dirty: bool,
}

impl Default for SpatialRegistry {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            dirty: true,
        }
    }
}

impl SpatialRegistry {
    pub fn at(&self, x: i32, y: i32) -> Option<Entity> {
        self.map.get(&(x, y)).copied()
    }

    pub fn is_free(&self, x: i32, y: i32) -> bool {
        !self.map.contains_key(&(x, y))
    }

    /// Returns all unique entities whose occupied tiles intersect the given rectangle.
    pub fn entities_in_rect(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<Entity> {
        let mut seen = Vec::new();
        for gx in x1..=x2 {
            for gy in y1..=y2 {
                if let Some(&entity) = self.map.get(&(gx, gy)) {
                    if !seen.contains(&entity) {
                        seen.push(entity);
                    }
                }
            }
        }
        seen
    }

    /// Returns true if ALL given tiles are free.
    pub fn tiles_are_free(&self, tiles: &[(i32, i32)]) -> bool {
        tiles.iter().all(|&(x, y)| self.is_free(x, y))
    }

    pub fn occupied_tiles(&self) -> impl Iterator<Item = &(i32, i32)> {
        self.map.keys()
    }
}

/// Rebuilds the spatial registry from all building OccupiedTiles.
/// Only rebuilds when buildings are added or removed.
pub fn sync_spatial_registry(
    mut registry: ResMut<SpatialRegistry>,
    query: Query<(Entity, &OccupiedTiles), With<Building>>,
    mut removals: RemovedComponents<OccupiedTiles>,
    added: Query<(), Added<OccupiedTiles>>,
) {
    let has_added = !added.is_empty();
    let has_removed = removals.read().count() > 0;
    if !registry.dirty && !has_added && !has_removed {
        return;
    }
    registry.dirty = false;
    registry.map.clear();
    registry.map.reserve(query.iter().len() * 2);
    for (entity, tiles) in &query {
        for &(x, y) in &tiles.0 {
            registry.map.insert((x, y), entity);
        }
    }
}
