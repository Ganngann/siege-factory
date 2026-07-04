use bevy::prelude::*;

use crate::economy::building::BuildingRegistry;
use crate::economy::components::{HQ, Building, OccupiedTiles, Active};
use crate::economy::resource::{ResourceId, Inventory};
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;

pub fn setup_hq(
    mut commands: Commands,
    hq_query: Query<Entity, With<HQ>>,
    cfg: Res<MapConfig>,
    registry: Res<BuildingRegistry>,
) {
    if !hq_query.is_empty() {
        return;
    }
    let def = registry.get("hq").expect("HQ building def missing");
    let tw = def.tile_size.0;
    let th = def.tile_size.1;
    let (bx, by) = cfg.hq_position;

    let mut occupied = Vec::with_capacity((tw * th) as usize);
    for dx in 0..tw {
        for dy in 0..th {
            occupied.push((bx + dx as i32, by + dy as i32));
        }
    }

    let cx = (bx as f32 + (tw as f32 - 1.0) * 0.5) * cfg.tile_size;
    let cy = (by as f32 + (th as f32 - 1.0) * 0.5) * cfg.tile_size;

    let mut inv = Inventory::new();
    inv.add(&ResourceId("ore".to_string()), cfg.hq_start_ore);

    commands.spawn((
        HQ,
        Building { kind: "hq".to_string(), name: "HQ".to_string() },
        inv,
        OccupiedTiles(occupied),
        Active(true),
        Transform::from_xyz(cx, cy, 1.0),
        Visibility::default(),
        TilePosition { x: bx, y: by },
    ));
}
