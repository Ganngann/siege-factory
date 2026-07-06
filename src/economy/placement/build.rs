use crate::agriculture::components::{Crop, Farm};
use crate::core::input::KeyBindings;
use crate::core::toast::ToastQueue;
use crate::economy::belt::BeltSlots;
use crate::economy::building::{BuildingRegistry, attach_power_components};
use crate::economy::components::{
    Archive, Assembler, BeltDirection, BuildMode, BuildPreview, Building, Direction,
    DiscoveredRecipes, Ghost, Miner, OccupiedTiles, ProductionCounter, ResourceDeposit,
    TurretCombat, UiIsBlocking, UnbuiltBuilding,
};
use crate::economy::game_components::Storage;
use crate::economy::resource::Inventory;
use crate::economy::spatial::SpatialRegistry;
use crate::events::DespawnDeposit;
use crate::map::components::{HoveredTile, TilePosition, cursor_to_tile};
use crate::map::config::MapConfig;
use crate::map::tile_grid::{CHUNK_SIZE, ChunkGrid};
use crate::rendering::{PreviewMaterials, ShapeCache, direction_arrow};
use bevy::prelude::*;

const BUILDING_TURRET: &str = "turret";
const BUILDING_STORAGE: &str = "storage";
const BUILDING_FARM: &str = "farm";
const BUILDING_ARCHIVE: &str = "archive";

// ── Auto-direction ──

fn detect_producer_direction(
    tx: i32,
    ty: i32,
    producers: &Query<&TilePosition, With<Miner>>,
) -> Option<Direction> {
    let offsets = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let dirs = [
        Direction::East,
        Direction::North,
        Direction::West,
        Direction::South,
    ];
    for (&(dx, dy), &dir) in offsets.iter().zip(dirs.iter()) {
        let nx = tx + dx;
        let ny = ty + dy;
        if producers.iter().any(|pos| pos.x == nx && pos.y == ny) {
            return Some(dir);
        }
    }
    None
}

pub fn auto_detect_direction(
    tx: i32,
    ty: i32,
    producers: &Query<&TilePosition, With<Miner>>,
    belts_query: &Query<(&TilePosition, &BeltSlots)>,
    default: Direction,
) -> Direction {
    if let Some(dir) = detect_producer_direction(tx, ty, producers) {
        return dir;
    }

    for (pos, slots) in belts_query.iter() {
        let (odx, ody) = slots.direction.offset();
        if pos.x + odx == tx && pos.y + ody == ty {
            return slots.direction;
        }
    }

    default
}

pub fn auto_detect_direction_from_data(
    tx: i32,
    ty: i32,
    producers: &Query<&TilePosition, With<Miner>>,
    belt_data: &[((i32, i32), Direction)],
    default: Direction,
) -> Direction {
    if let Some(dir) = detect_producer_direction(tx, ty, producers) {
        return dir;
    }

    for &((px, py), dir) in belt_data {
        let (odx, ody) = dir.offset();
        if px + odx == tx && py + ody == ty {
            return dir;
        }
    }

    default
}

// ── Click-drag helpers ──

pub fn compute_line(start: (i32, i32), end: (i32, i32)) -> Vec<(i32, i32, Direction)> {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let adx = dx.abs();
    let ady = dy.abs();

    if adx == 0 && ady == 0 {
        return vec![(start.0, start.1, Direction::East)];
    }

    let mut result = Vec::new();

    if adx > 0 && ady > 0 {
        let sdx = dx.signum();
        let sdy = dy.signum();
        let dir_x = if sdx > 0 {
            Direction::East
        } else {
            Direction::West
        };
        let dir_y = if sdy > 0 {
            Direction::North
        } else {
            Direction::South
        };

        if adx >= ady {
            for i in 0..adx {
                result.push((start.0 + sdx * i, start.1, dir_x));
            }
            for i in 0..=ady {
                result.push((end.0, start.1 + sdy * i, dir_y));
            }
        } else {
            for i in 0..ady {
                result.push((start.0, start.1 + sdy * i, dir_y));
            }
            for i in 0..=adx {
                result.push((start.0 + sdx * i, end.1, dir_x));
            }
        }
    } else if adx > 0 {
        let sdx = dx.signum();
        let dir = if sdx > 0 {
            Direction::East
        } else {
            Direction::West
        };
        for i in 0..=adx {
            result.push((start.0 + sdx * i, start.1, dir));
        }
    } else {
        let sdy = dy.signum();
        let dir = if sdy > 0 {
            Direction::North
        } else {
            Direction::South
        };
        for i in 0..=ady {
            result.push((start.0, start.1 + sdy * i, dir));
        }
    }

    result
}

// ── Multi-tile helpers ──

fn compute_footprint(tx: i32, ty: i32, tw: u32, th: u32) -> Vec<(i32, i32)> {
    let mut tiles = Vec::with_capacity((tw * th) as usize);
    for dx in 0..tw {
        for dy in 0..th {
            tiles.push((tx + dx as i32, ty + dy as i32));
        }
    }
    tiles
}

// ── Preview ──

#[allow(clippy::too_many_arguments)]
pub fn update_build_preview(
    mut commands: Commands,
    mut preview: ResMut<BuildPreview>,
    build_mode: Res<BuildMode>,
    belt_dir: Res<BeltDirection>,
    deconstruct: Res<crate::economy::components::DeconstructMode>,
    cfg: Res<MapConfig>,
    shapes: Res<ShapeCache>,
    preview_materials: Res<PreviewMaterials>,
    spatial: Res<SpatialRegistry>,
    deposits: Query<&TilePosition, With<ResourceDeposit>>,
    producers: Query<&TilePosition, With<Miner>>,
    belts_query: Query<(&TilePosition, &BeltSlots)>,
    registry: Res<BuildingRegistry>,
    hovered: Res<HoveredTile>,
    ghosts: Query<Entity, With<Ghost>>,
    drag: Res<crate::economy::components::BeltDrag>,
) {
    for entity in ghosts.iter() {
        commands.entity(entity).despawn();
    }
    preview.0 = None;

    if deconstruct.0 {
        let Some(TilePosition { x: tx, y: ty }) = hovered.0 else {
            return;
        };
        let occupied_here = !spatial.is_free(tx, ty);
        let mat = if occupied_here {
            preview_materials.deconstruct_building.clone()
        } else {
            preview_materials.deconstruct_zone.clone()
        };
        commands.spawn((
            Ghost,
            Mesh2d(shapes.rectangle.clone()),
            MeshMaterial2d(mat),
            Transform::from_xyz(tx as f32 * cfg.tile_size, ty as f32 * cfg.tile_size, 1.8),
        ));
        return;
    }

    let Some(ref kind) = build_mode.0 else { return };
    let Some(def) = registry.get(kind) else {
        return;
    };
    let Some(TilePosition { x: tx, y: ty }) = hovered.0 else {
        return;
    };

    // ── Drag line preview ──
    if def.belt.is_some() || def.drag_placement {
        if let Some((sx, sy)) = drag.start_coord {
            let line = compute_line((sx, sy), (tx, ty));
            for &(lx, ly, dir) in &line {
                let has_belt = belts_query.iter().any(|(p, _)| p.x == lx && p.y == ly);
                let valid = has_belt || spatial.is_free(lx, ly);
                let mat_handle = if valid {
                    preview_materials.build_valid.clone()
                } else {
                    preview_materials.build_invalid.clone()
                };
                let cx = lx as f32 * cfg.tile_size;
                let cy = ly as f32 * cfg.tile_size;
                if def.belt.is_some() {
                    let angle = match dir {
                        Direction::East => 0.0,
                        Direction::North => std::f32::consts::FRAC_PI_2,
                        Direction::West => std::f32::consts::PI,
                        Direction::South => -std::f32::consts::FRAC_PI_2,
                    };
                    commands.spawn((
                        Ghost,
                        Mesh2d(shapes.rectangle.clone()),
                        MeshMaterial2d(mat_handle),
                        Transform::from_xyz(cx, cy, 1.8)
                            .with_rotation(Quat::from_rotation_z(angle)),
                        Text2d::new(direction_arrow(dir).to_string()),
                        TextFont::from_font_size(18.0),
                        TextColor(if valid {
                            Color::srgba(0.0, 0.8, 0.0, 0.6)
                        } else {
                            Color::srgba(0.8, 0.0, 0.0, 0.5)
                        }),
                        TextLayout::justify(Justify::Center),
                    ));
                } else {
                    commands.spawn((
                        Ghost,
                        Mesh2d(shapes.rectangle.clone()),
                        MeshMaterial2d(mat_handle),
                        Transform::from_xyz(cx, cy, 1.8),
                    ));
                }
            }
            return;
        }
    }

    // ── Multi-tile preview ──
    let (tw, th) = def.tile_size;
    let footprint = compute_footprint(tx, ty, tw, th);

    let valid = if def.requires_deposit {
        deposits.iter().any(|pos| pos.x == tx && pos.y == ty) && spatial.tiles_are_free(&footprint)
    } else {
        spatial.tiles_are_free(&footprint)
    };

    let mat_handle = if valid {
        preview_materials.build_valid.clone()
    } else {
        preview_materials.build_invalid.clone()
    };
    let z = 1.8;
    let cx = (tx as f32 + (tw as f32 - 1.0) * 0.5) * cfg.tile_size;
    let cy = (ty as f32 + (th as f32 - 1.0) * 0.5) * cfg.tile_size;

    let entity = if def.belt.is_some() {
        let dir = auto_detect_direction(tx, ty, &producers, &belts_query, belt_dir.0);
        let angle = match dir {
            Direction::East => 0.0,
            Direction::North => std::f32::consts::FRAC_PI_2,
            Direction::West => std::f32::consts::PI,
            Direction::South => -std::f32::consts::FRAC_PI_2,
        };
        let text_color = if valid {
            Color::srgba(0.0, 0.8, 0.0, 0.6)
        } else {
            Color::srgba(0.8, 0.0, 0.0, 0.5)
        };
        let ghost_entity = commands
            .spawn((
                Ghost,
                Mesh2d(shapes.rectangle.clone()),
                MeshMaterial2d(mat_handle),
                Transform::from_xyz(cx, cy, z).with_rotation(Quat::from_rotation_z(angle)),
                Text2d::new(direction_arrow(dir).to_string()),
                TextFont::from_font_size(18.0),
                TextColor(text_color),
                TextLayout::justify(Justify::Center),
            ))
            .id();

        // Connection indicators
        let offsets = [(1, 0), (0, 1), (-1, 0), (0, -1)];
        let dirs = [
            Direction::East,
            Direction::North,
            Direction::West,
            Direction::South,
        ];
        for (&(dx, dy), &check_dir) in offsets.iter().zip(dirs.iter()) {
            let nx = tx + dx;
            let ny = ty + dy;
            let is_input = producers.iter().any(|pos| pos.x == nx && pos.y == ny)
                || belts_query.iter().any(|(pos, slots)| {
                    let (odx, ody) = slots.direction.offset();
                    pos.x + odx == tx && pos.y + ody == ty
                });
            if is_input || check_dir == dir {
                let indicator_mat = if is_input {
                    preview_materials.indicator_input.clone()
                } else {
                    preview_materials.indicator_output.clone()
                };
                let ix = cx + dx as f32 * cfg.tile_size * 0.4;
                let iy = cy + dy as f32 * cfg.tile_size * 0.4;
                commands.spawn((
                    Ghost,
                    Mesh2d(shapes.circle.clone()),
                    MeshMaterial2d(indicator_mat),
                    Transform::from_xyz(ix, iy, z + 0.1).with_scale(Vec3::splat(0.25)),
                ));
            }
        }

        ghost_entity
    } else {
        let mesh = shapes.get_visual(&def.visual);
        commands
            .spawn((
                Ghost,
                Mesh2d(mesh),
                MeshMaterial2d(mat_handle),
                Transform::from_xyz(cx, cy, z),
            ))
            .id()
    };

    preview.0 = Some(entity);
}

// ── Build click (blueprint placement) ──

#[allow(clippy::too_many_arguments)]
pub fn handle_build_click(
    mut commands: Commands,
    build_mode: Res<BuildMode>,
    cfg: Res<MapConfig>,
    spatial: Res<SpatialRegistry>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    deposits: Query<(Entity, &TilePosition, &ResourceDeposit)>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
    registry: Res<BuildingRegistry>,
    mut toast_queue: ResMut<ToastQueue>,
    mut chunk_grid: ResMut<ChunkGrid>,
    ui_blocking: Res<UiIsBlocking>,
    crops: Query<(Entity, &Crop, &Transform)>,
) {
    if ui_blocking.0 {
        return;
    }
    let tile_size = cfg.tile_size;

    let Some(ref kind) = build_mode.0 else { return };
    if !bindings.just_pressed("place", &keys, &buttons) {
        return;
    }

    let Some(TilePosition { x: tx, y: ty }) = cursor_to_tile(&windows, &camera, &cfg) else {
        return;
    };

    let def = match registry.get(kind) {
        Some(d) => d,
        None => return,
    };

    let (tw, th) = def.tile_size;

    // Buildings with belt properties or drag_placement are handled by track_belt_drag
    if def.belt.is_some() || def.drag_placement {
        return;
    }

    let footprint = compute_footprint(tx, ty, tw, th);

    if def.requires_deposit {
        let deposit_data = deposits
            .iter()
            .find(|(_, pos, _)| pos.x == tx && pos.y == ty);
        let Some((deposit_entity, _, res_dep)) = deposit_data else {
            toast_queue.0.push("No resource deposit here".to_string());
            return;
        };
        if !spatial.tiles_are_free(&footprint) {
            toast_queue.0.push("Tile already occupied".to_string());
            return;
        }

        if !cfg.infinite_deposits {
            let cx = tx.div_euclid(CHUNK_SIZE as i32);
            let cy = ty.div_euclid(CHUNK_SIZE as i32);
            let dx = tx.rem_euclid(CHUNK_SIZE as i32) as u32;
            let dy = ty.rem_euclid(CHUNK_SIZE as i32) as u32;
            chunk_grid.set_deposit_amount(cx, cy, dx, dy, 0);
            commands.trigger(DespawnDeposit(deposit_entity));
        }

        let cx = (tx as f32 + (tw as f32 - 1.0) * 0.5) * tile_size;
        let cy = (ty as f32 + (th as f32 - 1.0) * 0.5) * tile_size;
        let deposit_resource = res_dep.resource.clone();
        let mine_recipe = format!("mine_{}", deposit_resource);
        let interval = def
            .production
            .as_ref()
            .map(|p| p.interval_sec)
            .unwrap_or(2.0);
        let mut e = commands.spawn((
            UnbuiltBuilding,
            Miner,
            Building {
                kind: def.id.clone(),
                name: def.name.clone(),
            },
            Inventory::new(),
            OccupiedTiles(footprint),
            Transform::from_xyz(cx, cy, 2.0),
            TilePosition { x: tx, y: ty },
            Assembler {
                production_timer: 0.0,
                interval,
                recipe_id: mine_recipe,
            },
            ProductionCounter::default(),
            DiscoveredRecipes::default(),
        ));
        attach_power_components(&mut e, &def);
        return;
    }

    if !spatial.tiles_are_free(&footprint) {
        toast_queue.0.push("Tile occupied".to_string());
        return;
    }

    // Despawn any crops on the building footprint
    for (crop_entity, _, crop_tf) in crops.iter() {
        let ctx = (crop_tf.translation.x / tile_size).round() as i32;
        let cty = (crop_tf.translation.y / tile_size).round() as i32;
        if footprint.iter().any(|&(fx, fy)| fx == ctx && fy == cty) {
            commands.entity(crop_entity).try_despawn();
        }
    }

    let cx = (tx as f32 + (tw as f32 - 1.0) * 0.5) * tile_size;
    let cy = (ty as f32 + (th as f32 - 1.0) * 0.5) * tile_size;

    let base = (
        UnbuiltBuilding,
        Building {
            kind: def.id.clone(),
            name: def.name.clone(),
        },
        OccupiedTiles(footprint),
        TilePosition { x: tx, y: ty },
        Transform::from_xyz(cx, cy, 2.0),
    );

    let inv = if def.inventory_capacity > 0 {
        Inventory::with_capacity(def.inventory_capacity)
    } else {
        Inventory::new()
    };

    if let Some(default_recipe) = &def.default_recipe {
        let interval = def.production_interval.unwrap_or(2.0);
        let mut e = commands.spawn((
            base,
            Assembler {
                production_timer: 0.0,
                interval,
                recipe_id: default_recipe.clone(),
            },
            inv,
            ProductionCounter::default(),
            DiscoveredRecipes::default(),
        ));
        attach_power_components(&mut e, &def);
    } else if def.id == BUILDING_TURRET {
        let stats = def.combat.as_ref();
        let mut e = commands.spawn((
            base,
            inv,
            TurretCombat {
                damage: stats.map_or(0, |s| s.damage),
                range_sq: stats.map_or(0.0, |s| s.range),
                fire_interval: stats.map_or(0.0, |s| s.fire_rate_sec),
                timer: 0.0,
                projectile_speed: stats.map_or(0.0, |s| s.projectile_speed),
            },
        ));
        attach_power_components(&mut e, &def);
    } else if def.id == BUILDING_STORAGE {
        let mut e = commands.spawn((base, inv, Storage));
        attach_power_components(&mut e, &def);
    } else if def.id == BUILDING_FARM {
        let crop_types = def.crop_types.clone();
        let mut e = commands.spawn((
            base,
            inv,
            Farm {
                crop_index: 0,
                crop_types,
            },
            ProductionCounter::default(),
            DiscoveredRecipes::default(),
        ));
        attach_power_components(&mut e, &def);
    } else if def.id == BUILDING_ARCHIVE {
        let mut e = commands.spawn((base, inv, Archive));
        attach_power_components(&mut e, &def);
    } else {
        let mut e = commands.spawn((base, inv));
        attach_power_components(&mut e, &def);
    }
}
