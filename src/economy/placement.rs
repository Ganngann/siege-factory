use bevy::prelude::*;
use crate::economy::belt::{BeltSlots, compute_slot_positions};
use crate::economy::building::{BuildingCost, BuildingRegistry};
use crate::economy::components::{
    BuildMode, BeltDirection, BuildPreview,
    Building, Miner, Assembler, OreDeposit, Ghost, HQ, Produces, TurretCombat,
};
use crate::economy::resource::Inventory;
use crate::core::toast::ToastQueue;
use crate::events::DespawnDeposit;
use crate::map::components::{HoveredTile, TilePosition};
use crate::map::config::MapConfig;
use crate::rendering::{direction_arrow, material_from_color, ShapeCache};

pub fn build_mode_input(
    mut build_mode: ResMut<BuildMode>,
    mut belt_dir: ResMut<BeltDirection>,
    keys: Res<ButtonInput<KeyCode>>,
    cfg: Res<MapConfig>,
    mut placed_belts: Query<(&mut BeltSlots, &mut Text2d, &TilePosition)>,
    registry: Res<BuildingRegistry>,
    hovered: Res<HoveredTile>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let build_ids: Vec<&String> = registry.buildings.iter()
        .filter(|b| b.id != "hq")
        .map(|b| &b.id)
        .collect();
    for (i, key) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5].iter().enumerate() {
        if keys.just_pressed(*key) {
            if let Some(id) = build_ids.get(i) {
                build_mode.0 = match &build_mode.0 {
                    Some(current) if current == *id => None,
                    _ => Some((*id).clone()),
                };
            }
        }
    }

    if keys.just_pressed(KeyCode::KeyR) && build_mode.0.as_deref() == Some("belt") {
        if let Some(pos) = hovered.0 {
            let mut rotated = false;
            for (mut belt, mut text, tile_pos) in placed_belts.iter_mut() {
                if tile_pos.x == pos.x && tile_pos.y == pos.y {
                    belt.direction = belt.direction.next();
                    belt.slot_positions = compute_slot_positions(
                        tile_pos.x, tile_pos.y, belt.direction,
                        belt.slots.len() as u32, cfg.tile_size,
                    );
                    text.0 = direction_arrow(belt.direction).to_string();
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

    // Right-click cancel
    if buttons.just_pressed(MouseButton::Right) {
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
    buildings: Query<&TilePosition, With<Building>>,
    deposits: Query<&TilePosition, With<OreDeposit>>,
    registry: Res<BuildingRegistry>,
    hovered: Res<HoveredTile>,
) {
    let Some(ref kind) = build_mode.0 else {
        despawn_ghost(&mut commands, &mut preview);
        return;
    };

    let Some(def) = registry.get(kind) else {
        despawn_ghost(&mut commands, &mut preview);
        return;
    };

    let Some(TilePosition { x: tx, y: ty }) = hovered.0 else {
        despawn_ghost(&mut commands, &mut preview);
        return;
    };

    let valid = if def.requires_deposit {
        deposits.iter().any(|pos| pos.x == tx && pos.y == ty)
            && !buildings.iter().any(|pos| pos.x == tx && pos.y == ty)
    } else {
        tile_is_free(tx, ty, &buildings)
    };

    let color = if valid {
        Color::srgba(0.0, 0.8, 0.0, 0.4)
    } else {
        Color::srgba(0.8, 0.0, 0.0, 0.3)
    };
    let mat_handle = materials.add(color);
    let material = MeshMaterial2d(mat_handle);

    if let Some(entity) = preview.0.take() {
        commands.entity(entity).despawn();
    }

    let z = 1.8;
    let cx = tx as f32 * cfg.tile_size;
    let cy = ty as f32 * cfg.tile_size;

    let entity = if def.id == "belt" {
        let dir = belt_dir.0;
        let text_color = if valid {
            Color::srgba(0.0, 0.8, 0.0, 0.6)
        } else {
            Color::srgba(0.8, 0.0, 0.0, 0.5)
        };
        commands.spawn((
            Ghost,
            Text2d::new(direction_arrow(dir).to_string()),
            TextFont::from_font_size(24.0),
            TextColor(text_color),
            TextLayout::justify(Justify::Center),
            Transform::from_xyz(cx, cy, z),
        )).id()
    } else {
        let mesh = shapes.get_visual(&def.visual);
        commands.spawn((
            Ghost,
            Mesh2d(mesh),
            material,
            Transform::from_xyz(cx, cy, z),
        )).id()
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
    buildings: Query<&TilePosition, With<Building>>,
    mut hq_query: Query<&mut Inventory, With<HQ>>,
    buttons: Res<ButtonInput<MouseButton>>,
    registry: Res<BuildingRegistry>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    let tile_size = cfg.tile_size;
    let grid_w = cfg.width;
    let grid_h = cfg.height;

    let Some(ref kind) = build_mode.0 else { return };
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_transform)) = camera.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else { return };

    let tile_x = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let tile_y = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

    if tile_x < 0 || tile_y < 0 || tile_x >= grid_w as i32 || tile_y >= grid_h as i32 {
        toast_queue.0.push("Outside map".to_string());
        return;
    }

    let tx = tile_x as u32;
    let ty = tile_y as u32;

    let def = match registry.get(kind) {
        Some(d) => d,
        None => return,
    };

    if def.requires_deposit {
        let deposit_entity = deposits.iter().find(|(_, pos)| pos.x == tx && pos.y == ty).map(|(e, _)| e);
        let Some(deposit) = deposit_entity else {
            toast_queue.0.push("No ore deposit here".to_string());
            return;
        };
        let already_mined = buildings.iter().any(|pos| pos.x == tx && pos.y == ty);
        if already_mined {
            toast_queue.0.push("Tile already occupied".to_string());
            return;
        }

        let mut hq_inv = match hq_query.single_mut() {
            Ok(inv) => inv,
            Err(_) => return,
        };

        if !can_afford(&hq_inv, &def.cost) {
            toast_queue.0.push("Not enough ore".to_string());
            return;
        }

        deduct_cost(&mut hq_inv, &def.cost);
        commands.trigger(DespawnDeposit(deposit));

        let mesh = shapes.get_visual(&def.visual);
        commands.spawn((
            Miner { production_timer: 0.0, interval: def.production.as_ref().map(|p| p.interval_sec).unwrap_or(2.0) },
            Building { kind: def.id.clone(), name: def.name.clone() },
            Inventory::new(),
            Mesh2d(mesh),
            MeshMaterial2d(material_from_color(&mut materials, def.color)),
            Transform::from_xyz(tx as f32 * tile_size, ty as f32 * tile_size, 2.0),
            TilePosition { x: tx, y: ty },
            Produces { resource: def.production.as_ref().map(|p| p.resource).unwrap_or(crate::economy::resource::ResourceId::Ore), interval: def.production.as_ref().map(|p| p.interval_sec).unwrap_or(2.0), timer: 0.0 },
        ));
        return;
    }

    if !tile_is_free(tx, ty, &buildings) {
        toast_queue.0.push("Tile occupied".to_string());
        return;
    }

    let mut hq_inv = match hq_query.single_mut() {
        Ok(inv) => inv,
        Err(_) => return,
    };

    if !can_afford(&hq_inv, &def.cost) {
        toast_queue.0.push("Not enough ore".to_string());
        return;
    }

    deduct_cost(&mut hq_inv, &def.cost);

    let base = (
        Building { kind: def.id.clone(), name: def.name.clone() },
        Inventory::new(),
        TilePosition { x: tx, y: ty },
    );

    if def.id == "belt" {
        let dir = belt_dir.0;
        let cx = tx as f32 * tile_size;
        let cy = ty as f32 * tile_size;
        let num_slots = def.belt.as_ref().map_or(4, |b| b.slots);
        let speed = def.belt.as_ref().map_or(2.0, |b| b.speed);
        let slot_positions = compute_slot_positions(tx, ty, dir, num_slots, tile_size);
        let slots = vec![None; num_slots as usize];
        commands.spawn((
            base,
            BeltSlots { direction: dir, slots, slot_positions, speed },
            Text2d::new(direction_arrow(dir).to_string()),
            TextFont::from_font_size(24.0),
            TextColor(Color::WHITE),
            TextLayout::justify(Justify::Center),
            Transform::from_xyz(cx, cy, 2.0),
        ));
    } else if def.id == "assembler" {
        let mesh = shapes.get_visual(&def.visual);
        commands.spawn((
            base,
            Assembler { production_timer: 0.0, interval: 2.0 },
            Mesh2d(mesh),
            MeshMaterial2d(material_from_color(&mut materials, def.color)),
            Transform::from_xyz(tx as f32 * tile_size, ty as f32 * tile_size, 2.0)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
        ));
    } else if def.id == "turret" {
        let mesh = shapes.get_visual(&def.visual);
        let stats = def.combat.as_ref().expect("turret def missing combat");
        commands.spawn((
            base,
            TurretCombat {
                damage: stats.damage,
                range_sq: stats.range,
                fire_interval: stats.fire_rate_sec,
                timer: 0.0,
            },
            Mesh2d(mesh),
            MeshMaterial2d(material_from_color(&mut materials, def.color)),
            Transform::from_xyz(tx as f32 * tile_size, ty as f32 * tile_size, 2.0),
        ));
    } else {
        let mesh = shapes.get_visual(&def.visual);
        commands.spawn((
            base,
            Mesh2d(mesh),
            MeshMaterial2d(material_from_color(&mut materials, def.color)),
            Transform::from_xyz(tx as f32 * tile_size, ty as f32 * tile_size, 2.0),
        ));
    }
}
