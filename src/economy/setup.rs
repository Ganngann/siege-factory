use bevy::prelude::*;

use crate::economy::building::BuildingRegistry;
use crate::economy::components::{HQ, Building, OccupiedTiles, Active};
use crate::economy::resource::{ResourceId, Inventory};
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::TextureCache;

pub fn setup_hq(
    mut commands: Commands,
    hq_query: Query<Entity, With<HQ>>,
    cfg: Res<MapConfig>,
    registry: Res<BuildingRegistry>,
    textures: Res<TextureCache>,
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

    let stem = &def.texture_stem;
    let size = Vec2::new(cfg.tile_size * tw as f32, cfg.tile_size * th as f32);

    commands.spawn((
        HQ,
        Building { kind: "hq".to_string(), name: "HQ".to_string() },
        inv,
        OccupiedTiles(occupied),
        Active(true),
        Sprite {
            image: textures.base(stem),
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(cx, cy, 1.0),
        Visibility::default(),
        TilePosition { x: bx, y: by },
    )).with_children(|parent| {
        if let Some(tex) = textures.owner(stem) {
            parent.spawn((
                Sprite {
                    image: tex,
                    custom_size: Some(size),
                    color: Color::srgb(0.2, 0.4, 0.8),
                    ..default()
                },
                Transform::default(),
            ));
        }
        if let Some(tex) = textures.level(stem) {
            parent.spawn((
                Sprite {
                    image: tex,
                    custom_size: Some(size),
                    color: Color::srgb(0.2, 0.8, 0.2),
                    ..default()
                },
                Transform::default(),
            ));
        }
    });
}
