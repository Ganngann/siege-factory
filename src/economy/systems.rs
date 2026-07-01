use std::collections::HashMap;
use bevy::prelude::*;
use bevy::sprite::{Anchor, Mesh2dHandle};
use crate::economy::building::{BuildingCost, BuildingRegistry};
use crate::economy::recipe::RecipeRegistry;
use crate::economy::resource::{ResourceId, Inventory};
use crate::events::DespawnDeposit;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::{direction_arrow, material_from_color, ShapeCache};

#[derive(Component)]
pub struct HQ;

#[derive(Component)]
pub struct OreDeposit {
    pub amount: u32,
}

#[derive(Component)]
pub struct Miner {
    pub production_timer: f32,
    pub interval: f32,
}

#[derive(Component)]
pub struct Assembler {
    pub production_timer: f32,
    pub interval: f32,
}

#[derive(Component)]
pub struct Building {
    pub name: String,
}

#[derive(Component)]
pub struct Turret {
    pub fire_timer: f32,
}

#[derive(Component)]
pub struct Belt {
    pub direction: Direction,
}

#[derive(Component)]
pub struct BeltItem {
    pub path: Vec<Vec2>,
    pub distance_traveled: f32,
    pub speed: f32,
    pub resource: ResourceId,
    pub dest_tile: (u32, u32),
}

#[derive(Component)]
pub struct Ghost;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    East,
    North,
    West,
    South,
}

impl Direction {
    pub fn offset(&self) -> (i32, i32) {
        match self {
            Direction::East => (1, 0),
            Direction::North => (0, 1),
            Direction::West => (-1, 0),
            Direction::South => (0, -1),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Direction::East => Direction::North,
            Direction::North => Direction::West,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Direction::East => Color::srgb(0.6, 0.5, 0.4),
            Direction::North => Color::srgb(0.5, 0.6, 0.4),
            Direction::West => Color::srgb(0.4, 0.5, 0.6),
            Direction::South => Color::srgb(0.5, 0.4, 0.6),
        }
    }
}

#[derive(Resource, Default)]
pub struct BuildMode(pub Option<BuildKind>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildKind {
    Miner,
    Assembler,
    Belt,
    Wall,
    Turret,
}

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

#[derive(Resource, Default)]
pub struct BeltDirection(pub Direction);

#[derive(Resource, Default)]
pub struct BuildPreview(pub Option<Entity>);

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
    mut placed_belts: Query<(&mut Belt, &mut Text, &TilePosition)>,
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
            commands.spawn((
                Ghost,
                ColorMesh2dBundle {
                    mesh: Mesh2dHandle(shapes.square.clone()),
                    material,
                    transform: Transform::from_xyz(cx, cy, z),
                    ..default()
                },
                Text2dBundle {
                    text: Text::from_section(direction_arrow(dir), TextStyle {
                        font_size: 24.0,
                        color: Color::srgba(1.0, 1.0, 1.0, 0.6),
                        ..default()
                    }),
                    text_anchor: Anchor::Center,
                    transform: Transform::from_xyz(cx, cy, z + 0.1),
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
            commands.spawn((
                Belt { direction: dir },
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

// ── Belt chain tracing ──

fn trace_belt_output(
    building: TilePosition,
    belt_map: &HashMap<(u32, u32), Direction>,
    grid_w: u32,
    grid_h: u32,
) -> Option<(u32, u32)> {
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let ax = building.x.wrapping_add_signed(dx);
        let ay = building.y.wrapping_add_signed(dy);
        if let Some(&dir) = belt_map.get(&(ax, ay)) {
            return Some(trace_chain((ax, ay), dir, belt_map, grid_w, grid_h));
        }
    }
    None
}

fn trace_chain(
    start: (u32, u32),
    start_dir: Direction,
    belt_map: &HashMap<(u32, u32), Direction>,
    grid_w: u32,
    grid_h: u32,
) -> (u32, u32) {
    let mut cur = start;
    let mut dir = start_dir;
    loop {
        let (dx, dy) = dir.offset();
        let nx = cur.0.wrapping_add_signed(dx);
        let ny = cur.1.wrapping_add_signed(dy);
        if nx >= grid_w || ny >= grid_h {
            return (nx.min(grid_w - 1), ny.min(grid_h - 1));
        }
        if let Some(&next_dir) = belt_map.get(&(nx, ny)) {
            cur = (nx, ny);
            dir = next_dir;
        } else {
            return (nx, ny);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn production_tick(
    time: Res<Time>,
    cfg: Res<MapConfig>,
    mut miner_query: Query<(&mut Miner, &TilePosition, &Transform)>,
    mut inventories: Query<(Entity, &TilePosition, &mut Inventory)>,
    belts: Query<(&TilePosition, &Belt)>,
    mut commands: Commands,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let grid_w = cfg.width;
    let grid_h = cfg.height;
    let tile_size = cfg.tile_size;

    let belt_map: HashMap<(u32, u32), Direction> =
        belts.iter().map(|(pos, belt)| ((pos.x, pos.y), belt.direction)).collect();

    let inv_lookup: HashMap<(u32, u32), Entity> =
        inventories.iter().map(|(e, pos, _)| ((pos.x, pos.y), e)).collect();

    for (mut miner, tile_pos, transform) in miner_query.iter_mut() {
        miner.production_timer += time.delta_seconds();
        while miner.production_timer >= miner.interval {
            miner.production_timer -= miner.interval;

            if let Some((end, tile_path)) = trace_belt_output_with_path(*tile_pos, &belt_map, grid_w, grid_h) {
                let world_path: Vec<Vec2> = tile_path.iter().map(|(x, y)| {
                    Vec2::new(*x as f32 * tile_size, *y as f32 * tile_size)
                }).collect();
                commands.spawn((
                    BeltItem {
                        path: world_path,
                        distance_traveled: 0.0,
                        speed: tile_size * 2.0,
                        resource: ResourceId::Ore,
                        dest_tile: end,
                    },
                    ColorMesh2dBundle {
                        mesh: Mesh2dHandle(shapes.circle.clone()),
                        material: material_from_color(&mut materials, Color::srgb(0.7, 0.5, 0.1)),
                        transform: Transform::from_xyz(transform.translation.x, transform.translation.y, 2.5)
                            .with_scale(Vec3::splat(0.25)),
                        ..default()
                    },
                ));
            } else {
                let end = trace_belt_output(*tile_pos, &belt_map, grid_w, grid_h)
                    .unwrap_or((tile_pos.x, tile_pos.y));
                if let Some(&target) = inv_lookup.get(&end)
                    && let Ok((_, _, mut inv)) = inventories.get_mut(target) {
                    inv.add(ResourceId::Ore, 1);
                }
            }
        }
    }
}

pub fn assembler_tick(
    time: Res<Time>,
    cfg: Res<MapConfig>,
    recipes: Res<RecipeRegistry>,
    mut assembler_query: Query<(&mut Assembler, &TilePosition)>,
    mut inventories: Query<(Entity, &TilePosition, &mut Inventory)>,
    belts: Query<(&TilePosition, &Belt)>,
) {
    let grid_w = cfg.width;
    let grid_h = cfg.height;

    let recipe = match recipes.get("ammo_craft") {
        Some(r) => r,
        None => return,
    };

    let belt_map: HashMap<(u32, u32), Direction> =
        belts.iter().map(|(pos, belt)| ((pos.x, pos.y), belt.direction)).collect();

    let inv_lookup: HashMap<(u32, u32), Entity> =
        inventories.iter().map(|(e, pos, _)| ((pos.x, pos.y), e)).collect();

    for (mut assembler, tile_pos) in assembler_query.iter_mut() {
        assembler.production_timer += time.delta_seconds();
        while assembler.production_timer >= recipe.time_sec {
            // Find input source: check belt chain for incoming resources
            let source = find_input_source(*tile_pos, &belt_map)
                .unwrap_or((tile_pos.x, tile_pos.y));
            let has_input = inv_lookup.get(&source)
                .and_then(|&e| inventories.get(e).ok())
                .map(|(_, _, inv)| inv.get(recipe.input_resource) >= recipe.input_amount)
                .unwrap_or(false);

            if !has_input {
                break;
            }

            // Consume input
            if let Some(&src_entity) = inv_lookup.get(&source)
                && let Ok((_, _, mut src_inv)) = inventories.get_mut(src_entity) {
                src_inv.remove(recipe.input_resource, recipe.input_amount);
            }

            // Output to belt or direct
            let output = trace_belt_output(*tile_pos, &belt_map, grid_w, grid_h)
                .unwrap_or((tile_pos.x, tile_pos.y));
            if let Some(&target) = inv_lookup.get(&output)
                && let Ok((_, _, mut inv)) = inventories.get_mut(target) {
                inv.add(recipe.output_resource, recipe.output_amount);
            }

            assembler.production_timer -= recipe.time_sec;
        }
    }
}

fn find_input_source(
    building: TilePosition,
    belt_map: &HashMap<(u32, u32), Direction>,
) -> Option<(u32, u32)> {
    // Check adjacent tiles for a belt pointing TOWARD the building
    let dirs: [(i32, i32, Direction); 4] = [
        (1, 0, Direction::West),
        (-1, 0, Direction::East),
        (0, 1, Direction::South),
        (0, -1, Direction::North),
    ];
    for (dx, dy, expected_dir) in dirs {
        let ax = building.x.wrapping_add_signed(dx);
        let ay = building.y.wrapping_add_signed(dy);
        if let Some(&dir) = belt_map.get(&(ax, ay))
            && dir == expected_dir {
            return Some(trace_chain_back((ax, ay), belt_map));
        }
    }
    None
}

pub fn move_belt_items(
    time: Res<Time>,
    mut commands: Commands,
    mut belt_items: Query<(Entity, &mut Transform, &mut BeltItem)>,
    mut inventories: Query<(Entity, &TilePosition, &mut Inventory)>,
) {
    let inv_lookup: HashMap<(u32, u32), Entity> =
        inventories.iter().map(|(e, pos, _)| ((pos.x, pos.y), e)).collect();

    let mut to_despawn = Vec::new();
    let dt = time.delta_seconds();

    for (entity, mut transform, mut item) in belt_items.iter_mut() {
        if item.path.len() < 2 {
            to_despawn.push(entity);
            continue;
        }

        item.distance_traveled += item.speed * dt;

        let total_dist: f32 = item.path.windows(2)
            .map(|w| w[0].distance(w[1]))
            .sum();

        if item.distance_traveled >= total_dist {
            if let Some(&target) = inv_lookup.get(&item.dest_tile)
                && let Ok((_, _, mut inv)) = inventories.get_mut(target) {
                inv.add(item.resource, 1);
            }
            to_despawn.push(entity);
        } else {
            let mut accumulated = 0.0;
            let mut pos = item.path[0];
            for i in 0..item.path.len() - 1 {
                let seg_len = item.path[i].distance(item.path[i + 1]);
                if item.distance_traveled <= accumulated + seg_len {
                    let t = if seg_len > 0.0 { (item.distance_traveled - accumulated) / seg_len } else { 0.0 };
                    pos = item.path[i].lerp(item.path[i + 1], t);
                    break;
                }
                accumulated += seg_len;
                pos = item.path[i + 1];
            }
            transform.translation = Vec3::new(pos.x, pos.y, 2.5);
        }
    }

    for entity in to_despawn {
        commands.entity(entity).despawn();
    }
}

fn trace_chain_back(
    start: (u32, u32),
    belt_map: &HashMap<(u32, u32), Direction>,
) -> (u32, u32) {
    let mut cur = start;
    loop {
        // Check if adjacent tile has a belt pointing INTO cur
        let mut found = false;
        for (dx, dy, expected) in [(1, 0, Direction::West), (-1, 0, Direction::East), (0, 1, Direction::South), (0, -1, Direction::North)] {
            let ax = cur.0.wrapping_add_signed(dx);
            let ay = cur.1.wrapping_add_signed(dy);
            if let Some(&dir) = belt_map.get(&(ax, ay))
                && dir == expected {
                cur = (ax, ay);
                found = true;
                break;
            }
        }
        if !found {
            return cur;
        }
    }
}

fn trace_chain_path(
    start: (u32, u32),
    start_dir: Direction,
    belt_map: &HashMap<(u32, u32), Direction>,
    grid_w: u32,
    grid_h: u32,
) -> Vec<(u32, u32)> {
    let mut path = vec![start];
    let mut cur = start;
    let mut dir = start_dir;
    loop {
        let (dx, dy) = dir.offset();
        let nx = cur.0.wrapping_add_signed(dx);
        let ny = cur.1.wrapping_add_signed(dy);
        if nx >= grid_w || ny >= grid_h {
            path.push((nx.min(grid_w - 1), ny.min(grid_h - 1)));
            return path;
        }
        if let Some(&next_dir) = belt_map.get(&(nx, ny)) {
            cur = (nx, ny);
            dir = next_dir;
            path.push(cur);
        } else {
            path.push((nx, ny));
            return path;
        }
    }
}

type BeltPath = Option<((u32, u32), Vec<(u32, u32)>)>;

fn trace_belt_output_with_path(
    building: TilePosition,
    belt_map: &HashMap<(u32, u32), Direction>,
    grid_w: u32,
    grid_h: u32,
) -> BeltPath {
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let ax = building.x.wrapping_add_signed(dx);
        let ay = building.y.wrapping_add_signed(dy);
        if let Some(&dir) = belt_map.get(&(ax, ay)) {
            let path = trace_chain_path((ax, ay), dir, belt_map, grid_w, grid_h);
            let end = *path.last().unwrap_or(&(ax, ay));
            return Some((end, path));
        }
    }
    None
}
