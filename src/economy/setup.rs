use bevy::prelude::*;

use crate::economy::building::BuildingRegistry;
use crate::economy::components::{HQ, OreDeposit, Building, OccupiedTiles};
use crate::economy::resource::{ResourceId, Inventory};
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::ShapeCache;

pub fn setup_hq(
    mut commands: Commands,
    hq_query: Query<Entity, With<HQ>>,
    cfg: Res<MapConfig>,
    registry: Res<BuildingRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
    let mesh = meshes.add(Rectangle::new(
        cfg.tile_size * tw as f32 - 4.0,
        cfg.tile_size * th as f32 - 4.0,
    ));
    commands.spawn((
        HQ,
        Building { kind: "hq".to_string(), name: "HQ".to_string() },
        inv,
        OccupiedTiles(occupied),
        Mesh2d(mesh),
        MeshMaterial2d(materials.add(Color::srgb(0.2, 0.5, 0.8))),
        Transform::from_xyz(cx, cy, 1.0),
        TilePosition { x: bx, y: by },
    ));
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
