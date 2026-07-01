use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use crate::economy::components::{HQ, OreDeposit, Building};
use crate::economy::resource::{ResourceId, Inventory};
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::{material_from_color, ShapeCache};

pub fn setup_hq(
    mut commands: Commands,
    hq_query: Query<Entity, With<HQ>>,
    cfg: Res<MapConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !hq_query.is_empty() {
        return;
    }
    let mut inv = Inventory::new();
    inv.add(ResourceId::Ore, cfg.hq_start_ore);
    let cx = cfg.width as f32 * cfg.tile_size / 2.0;
    let cy = cfg.height as f32 * cfg.tile_size / 2.0;
    commands.spawn((
        HQ,
        Building { name: "HQ".to_string() },
        inv,
        ColorMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(cfg.tile_size * 2.0 - 4.0, cfg.tile_size * 2.0 - 4.0))),
            material: materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.5, 0.8))),
            transform: Transform::from_xyz(cx, cy, 1.0),
            ..default()
        },
        TilePosition { x: cfg.width / 2, y: cfg.height / 2 },
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

    let material = material_from_color(&mut materials, Color::srgb(0.7, 0.5, 0.1));

    for &(x, y) in &cfg.deposit_positions {
        commands.spawn((
            OreDeposit { amount: cfg.deposit_max_amount },
            ColorMesh2dBundle {
                mesh: Mesh2dHandle(shapes.circle.clone()),
                material: material.clone(),
                transform: Transform::from_xyz(x as f32 * cfg.tile_size, y as f32 * cfg.tile_size, 0.5),
                ..default()
            },
            TilePosition { x, y },
        ));
    }
}
