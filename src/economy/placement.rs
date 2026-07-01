use bevy::prelude::*;
use bevy::sprite::{Anchor, Mesh2dHandle};
use crate::economy::belt::{BeltSlots, compute_slot_positions};
use crate::economy::building::{BuildingCost, BuildingRegistry};
use crate::economy::components::{
    BuildKind, BuildMode, BeltDirection, BuildPreview,
    SetBuildModeEvent, Building, Miner, Assembler, Turret, OreDeposit, Ghost, HQ,
};
use crate::economy::resource::Inventory;
use crate::events::DespawnDeposit;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::{direction_arrow, material_from_color, ShapeCache};

fn cursor_tile(
    windows: &Query<&Window>,
    camera: &Query<(&Camera, &GlobalTransform)>,
    cfg: &MapConfig,
) -> Option<(u32, u32)> {
    let window = windows.iter().next()?;
    let cursor = window.cursor_position()?;
    let (cam, cam_transform) = camera.iter().next()?;
    let world_pos = cam.viewport_to_world_2d(cam_transform, cursor)?;
    let tile_x = ((world_pos.x + cfg.tile_size / 2.0) / cfg.tile_size).floor() as i32;
    let tile_y = ((world_pos.y + cfg.tile_size / 2.0) / cfg.tile_size).floor() as i32;
    if tile_x < 0 || tile_y < 0 || tile_x >= cfg.width as i32 || tile_y >= cfg.height as i32 {
        return None;
    }
    Some((tile_x as u32, tile_y as u32))
}

pub fn build_mode_input(
    mut build_mode: ResMut<BuildMode>,
    mut belt_dir: ResMut<BeltDirection>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    cfg: Res<MapConfig>,
    mut placed_belts: Query<(&mut BeltSlots, &mut Text, &TilePosition)>,
    mut mode_events: EventReader<SetBuildModeEvent>,
) {
    let key_map = [
        (KeyCode::Digit1, BuildKind::Miner),
        (KeyCode::Digit2, BuildKind::Assembler),
        (KeyCode::Digit3, BuildKind::Belt),
        (KeyCode::Digit4, BuildKind::Wall),
        (KeyCode::Digit5, BuildKind::Turret),
    ];

    for (key, kind) in key_map {
        if keys.just_pressed(key) {
            build_mode.0 = match build_mode.0 {
                Some(k) if k == kind => None,
                _ => Some(kind),
            };
        }
    }

    if keys.just_pressed(KeyCode::KeyR) && build_mode.0 == Some(BuildKind::Belt) {
        if let Some((tx, ty)) = cursor_tile(&windows, &camera, &cfg) {
            let mut rotated = false;
            for (mut belt, mut text, pos) in placed_belts.iter_mut() {
                if pos.x == tx && pos.y == ty {
                    belt.direction = belt.direction.next();
                    belt.slot_positions = compute_slot_positions(
                        pos.x, pos.y, belt.direction,
                        belt.slots.len() as u32, cfg.tile_size,
                    );
                    text.sections[0].value = direction_arrow(belt.direction).to_string();
                    rotated = true;
                    break;
                }
            }
            if !rotated {
                belt_dir.0 = belt_dir.0.next();
            }
        } else {
            belt_dir.0 = belt_dir.0.next();
        }
    }

    for ev in mode_events.read() {
        build_mode.0 = ev.0;
    }

    if keys.just_pressed(KeyCode::Escape) {
        build_mode.0 = None;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_build_preview(
    mut commands: Commands,
    mut preview: ResMut<BuildPreview>,
    build_mode: Res<BuildMode>,
    belt_dir: Res<BeltDirection>,
    cfg: Res<MapConfig>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    buildings: Query<&TilePosition, With<Building>>,
    deposits: Query<&TilePosition, With<OreDeposit>>,
    miners: Query<&TilePosition, With<Miner>>,
) {
    let Some(kind) = build_mode.0 else {
        despawn_ghost(&mut commands, &mut preview);
        return;
    };

    let Some((tx, ty)) = cursor_tile(&windows, &camera, &cfg) else {
        despawn_ghost(&mut commands, &mut preview);
        return;
    };

    let valid = match kind {
        BuildKind::Miner => {
            deposits.iter().any(|pos| pos.x == tx && pos.y == ty)
                && !miners.iter().any(|pos| pos.x == tx && pos.y == ty)
        }
        _ => tile_is_free(tx, ty, &buildings),
    };

    let color = if valid {
        Color::srgba(0.0, 0.8, 0.0, 0.4)
    } else {
        Color::srgba(0.8, 0.0, 0.0, 0.3)
    };
    let material = materials.add(ColorMaterial::from_color(color));

    if let Some(entity) = preview.0.take() {
        commands.entity(entity).despawn();
    }

    let z = 1.8;
    let cx = tx as f32 * cfg.tile_size;
    let cy = ty as f32 * cfg.tile_size;

    let entity = match kind {
        BuildKind::Miner => commands.spawn((
            Ghost,
            ColorMesh2dBundle {
                mesh: Mesh2dHandle(shapes.square.clone()),
                material,
                transform: Transform::from_xyz(cx, cy, z),
                ..default()
            },
        )).id(),
        BuildKind::Assembler => commands.spawn((
            Ghost,
            ColorMesh2dBundle {
                mesh: Mesh2dHandle(shapes.diamond.clone()),
                material,
                transform: Transform::from_xyz(cx, cy, z)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
                ..default()
            },
        )).id(),
        BuildKind::Belt => {
            let dir = belt_dir.0;
            let text_color = if valid {
                Color::srgba(0.0, 0.8, 0.0, 0.6)
            } else {
                Color::srgba(0.8, 0.0, 0.0, 0.5)
            };
            commands.spawn((
                Ghost,
                Text2dBundle {
                    text: Text::from_section(direction_arrow(dir), TextStyle {
                        font_size: 24.0,
                        color: text_color,
                        ..default()
                    }),
                    text_anchor: Anchor::Center,
                    transform: Transform::from_xyz(cx, cy, z),
                    ..default()
                },
            )).id()
        }
        BuildKind::Wall => commands.spawn((
            Ghost,
            ColorMesh2dBundle {
                mesh: Mesh2dHandle(shapes.rectangle.clone()),
                material,
                transform: Transform::from_xyz(cx, cy, z),
                ..default()
            },
        )).id(),
        BuildKind::Turret => commands.spawn((
            Ghost,
            ColorMesh2dBundle {
                mesh: Mesh2dHandle(shapes.triangle.clone()),
                material,
                transform: Transform::from_xyz(cx, cy, z),
                ..default()
            },
        )).id(),
    };

    preview.0 = Some(entity);
}

fn despawn_ghost(commands: &mut Commands, preview: &mut ResMut<BuildPreview>) {
    if let Some(entity) = preview.0.take() {
        commands.entity(entity).despawn();
    }
}

fn can_afford(hq_inv: &Inventory, cost: &[BuildingCost]) -> bool {
    cost.iter().all(|c| hq_inv.get(c.resource) >= c.amount)
}

fn deduct_cost(hq_inv: &mut Inventory, cost: &[BuildingCost]) {
    for c in cost {
        hq_inv.remove(c.resource, c.amount);
    }
}

fn tile_is_free(tx: u32, ty: u32, buildings: &Query<&TilePosition, With<Building>>) -> bool {
    !buildings.iter().any(|pos| pos.x == tx && pos.y == ty)
}

#[allow(clippy::too_many_arguments)]
pub fn handle_build_click(
    mut commands: Commands,
    build_mode: Res<BuildMode>,
    belt_dir: Res<BeltDirection>,
    cfg: Res<MapConfig>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    deposits: Query<(Entity, &TilePosition), With<OreDeposit>>,
    miners: Query<&TilePosition, With<Miner>>,
    buildings: Query<&TilePosition, With<Building>>,
    mut hq_query: Query<&mut Inventory, (With<HQ>, Without<Miner>)>,
    buttons: Res<ButtonInput<MouseButton>>,
    registry: Res<BuildingRegistry>,
    mut deposit_events: EventWriter<DespawnDeposit>,
) {
    let tile_size = cfg.tile_size;
    let grid_w = cfg.width;
    let grid_h = cfg.height;

    let Some(kind) = build_mode.0 else { return };
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let window = windows.single();
    let (cam, cam_transform) = camera.single();
    let Some(cursor) = window.cursor_position() else { return };
    let Some(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else { return };

    let tile_x = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let tile_y = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

    if tile_x < 0 || tile_y < 0 || tile_x >= grid_w as i32 || tile_y >= grid_h as i32 {
        return;
    }

    let tx = tile_x as u32;
    let ty = tile_y as u32;

    if kind == BuildKind::Miner {
        let deposit_entity = deposits.iter().find(|(_, pos)| pos.x == tx && pos.y == ty).map(|(e, _)| e);
        let Some(deposit) = deposit_entity else { return };

        let already_mined = miners.iter().any(|pos| pos.x == tx && pos.y == ty);
        if already_mined {
            return;
        }

        let def = match registry.get("miner") {
            Some(d) => d,
            None => return,
        };

        let mut hq_inv = match hq_query.get_single_mut() {
            Ok(inv) => inv,
            Err(_) => return,
        };

        if !can_afford(&hq_inv, &def.cost) {
            return;
        }

        deduct_cost(&mut hq_inv, &def.cost);
        deposit_events.send(DespawnDeposit(deposit));
        commands.spawn((
            Miner { production_timer: 0.0, interval: 2.0 },
            Building { name: def.name.clone() },
            Inventory::new(),
            ColorMesh2dBundle {
                mesh: Mesh2dHandle(shapes.square.clone()),
                material: material_from_color(&mut materials, def.color),
                transform: Transform::from_xyz(tx as f32 * tile_size, ty as f32 * tile_size, 2.0),
                ..default()
            },
            TilePosition { x: tx, y: ty },
        ));
        return;
    }

    let building_id = match kind {
        BuildKind::Assembler => "assembler",
        BuildKind::Belt => "belt",
        BuildKind::Wall => "wall",
        BuildKind::Turret => "turret",
        _ => return,
    };

    let def = match registry.get(building_id) {
        Some(d) => d,
        None => return,
    };

    if !tile_is_free(tx, ty, &buildings) {
        return;
    }

    let mut hq_inv = match hq_query.get_single_mut() {
        Ok(inv) => inv,
        Err(_) => return,
    };

    if !can_afford(&hq_inv, &def.cost) {
        return;
    }

    deduct_cost(&mut hq_inv, &def.cost);

    match kind {
        BuildKind::Assembler => {
            commands.spawn((
                Assembler { production_timer: 0.0, interval: 2.0 },
                Building { name: def.name.clone() },
                Inventory::new(),
                ColorMesh2dBundle {
                    mesh: Mesh2dHandle(shapes.diamond.clone()),
                    material: material_from_color(&mut materials, def.color),
                    transform: Transform::from_xyz(tx as f32 * tile_size, ty as f32 * tile_size, 2.0)
                        .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
                    ..default()
                },
                TilePosition { x: tx, y: ty },
            ));
        }
        BuildKind::Belt => {
            let dir = belt_dir.0;
            let cx = tx as f32 * tile_size;
            let cy = ty as f32 * tile_size;
            let num_slots = def.belt.as_ref().map_or(4, |b| b.slots);
            let speed = def.belt.as_ref().map_or(2.0, |b| b.speed);
            let slot_positions = compute_slot_positions(tx, ty, dir, num_slots, tile_size);
            let slots = vec![None; num_slots as usize];
            commands.spawn((
                BeltSlots { direction: dir, slots, slot_positions, speed },
                Building { name: def.name.clone() },
                Inventory::new(),
                Text2dBundle {
                    text: Text::from_section(direction_arrow(dir), TextStyle { font_size: 24.0, color: Color::WHITE, ..default() }),
                    text_anchor: Anchor::Center,
                    transform: Transform::from_xyz(cx, cy, 2.0),
                    ..default()
                },
                TilePosition { x: tx, y: ty },
            ));
        }
        BuildKind::Wall => {
            commands.spawn((
                Building { name: def.name.clone() },
                Inventory::new(),
                ColorMesh2dBundle {
                    mesh: Mesh2dHandle(shapes.rectangle.clone()),
                    material: material_from_color(&mut materials, def.color),
                    transform: Transform::from_xyz(tx as f32 * tile_size, ty as f32 * tile_size, 2.0),
                    ..default()
                },
                TilePosition { x: tx, y: ty },
            ));
        }
        BuildKind::Turret => {
            commands.spawn((
                Turret { fire_timer: 0.0 },
                Building { name: def.name.clone() },
                Inventory::new(),
                ColorMesh2dBundle {
                    mesh: Mesh2dHandle(shapes.triangle.clone()),
                    material: material_from_color(&mut materials, def.color),
                    transform: Transform::from_xyz(tx as f32 * tile_size, ty as f32 * tile_size, 2.0),
                    ..default()
                },
                TilePosition { x: tx, y: ty },
            ));
        }
        _ => {}
    }
}
