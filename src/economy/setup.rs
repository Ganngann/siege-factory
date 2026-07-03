use bevy::prelude::*;

use crate::economy::building::BuildingRegistry;
use crate::economy::components::{HQ, OreDeposit, Building, OccupiedTiles};
use crate::economy::resource::{ResourceId, Inventory};
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::{ShapeCache, TextureCache, texture_stem};

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
            occupied.push((bx + dx, by + dy));
        }
    }

    let cx = (bx as f32 + (tw as f32 - 1.0) * 0.5) * cfg.tile_size;
    let cy = (by as f32 + (th as f32 - 1.0) * 0.5) * cfg.tile_size;

    let mut inv = Inventory::new();
    inv.add(ResourceId::Ore, cfg.hq_start_ore);

    let stem = texture_stem("hq");
    let size = Vec2::new(cfg.tile_size * tw as f32, cfg.tile_size * th as f32);

    commands.spawn((
        HQ,
        Building { kind: "hq".to_string(), name: "HQ".to_string() },
        inv,
        OccupiedTiles(occupied),
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

pub fn place_ore_deposits(
    mut commands: Commands,
    deposit_query: Query<Entity, With<OreDeposit>>,
    cfg: Res<MapConfig>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !deposit_query.is_empty() {
        return;
    }

    let mat = materials.add(Color::srgb(0.7, 0.5, 0.1));

    for &(x, y) in &cfg.deposit_positions {
        commands.spawn((
            OreDeposit { amount: cfg.deposit_max_amount },
            Mesh2d(shapes.circle.clone()),
            MeshMaterial2d(mat.clone()),
            Transform::from_xyz(x as f32 * cfg.tile_size, y as f32 * cfg.tile_size, 0.5),
            TilePosition { x, y },
        ));
    }
}
