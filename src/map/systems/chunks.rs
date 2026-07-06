use crate::core::utils::tile_to_world_corner;
use crate::economy::components::{PeacefulMode, ResourceDeposit};
use crate::economy::resource::ResourceRegistry;
use crate::map::components::{ChunkMember, Decoration, TilePosition};
use crate::map::config::MapConfig;
use crate::map::rng::{SimpleRng, chunk_hash};
use crate::map::tile_grid::{CHUNK_SIZE, ChunkGrid};
use crate::rendering::{ShapeCache, TextureCache};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::{
    Assets, Camera, Color, ColorMaterial, Commands, Entity, GlobalTransform, Mesh, Mesh2d,
    MeshMaterial2d, Query, Res, ResMut, Sprite, Transform, Vec2, Window, default,
};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use std::collections::HashSet;

pub fn build_chunk_mesh(cx: i32, cy: i32, tile_size: f32) -> (Mesh, Mesh) {
    let chunk_size = CHUNK_SIZE as i32;
    let world_ox = cx * chunk_size;
    let world_oy = cy * chunk_size;

    let mut pos_even = Vec::new();
    let mut pos_odd = Vec::new();
    let mut idx_even = Vec::new();
    let mut idx_odd = Vec::new();
    let mut n_even = 0u32;
    let mut n_odd = 0u32;

    for ty in 0..CHUNK_SIZE as usize {
        for tx in 0..CHUNK_SIZE as usize {
            let wx = world_ox + tx as i32;
            let wy = world_oy + ty as i32;
            let pos = tile_to_world_corner(wx, wy, tile_size);
            let (x, y) = (pos.x, pos.y);
            let s = tile_size;

            let quad_positions = [
                [x, y, 0.0],
                [x + s, y, 0.0],
                [x + s, y + s, 0.0],
                [x, y + s, 0.0],
            ];
            let quad_indices =
                |base: u32| -> [u32; 6] { [base, base + 1, base + 2, base, base + 2, base + 3] };

            if (wx + wy) % 2 == 0 {
                pos_even.extend_from_slice(&quad_positions);
                idx_even.extend_from_slice(&quad_indices(n_even));
                n_even += 4;
            } else {
                pos_odd.extend_from_slice(&quad_positions);
                idx_odd.extend_from_slice(&quad_indices(n_odd));
                n_odd += 4;
            }
        }
    }

    let mesh_a = mesh_from_quads(pos_even, idx_even);
    let mesh_b = mesh_from_quads(pos_odd, idx_odd);
    (mesh_a, mesh_b)
}

fn mesh_from_quads(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> Mesh {
    let normals = vec![[0.0, 0.0, 1.0]; positions.len()];
    let uvs = vec![[0.0, 0.0]; positions.len()];

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn spawn_single_chunk_visuals(
    commands: &mut Commands,
    chunk_grid: &mut ChunkGrid,
    cfg: &MapConfig,
    res_registry: &ResourceRegistry,
    shapes: &ShapeCache,
    materials: &mut Assets<ColorMaterial>,
    meshes: &mut Assets<Mesh>,
    textures: &TextureCache,
    cx: i32,
    cy: i32,
) {
    let chunk_size = CHUNK_SIZE as i32;
    let tile_size = cfg.tile_size;

    let chunk_hash = chunk_hash(cfg.seed, cx, cy);
    let (mesh_even, mesh_odd) = build_chunk_mesh(cx, cy, tile_size);
    let chunk = chunk_grid.ensure_chunk(cx, cy);

    let mat_even = materials.add(Color::srgb(0.25, 0.35, 0.25));
    let mat_odd = materials.add(Color::srgb(0.18, 0.28, 0.18));

    commands.spawn(super::ChunkMarker(cx, cy));
    commands.spawn((
        ChunkMember(cx, cy),
        Mesh2d(meshes.add(mesh_even)),
        MeshMaterial2d(mat_even),
        Transform::default(),
    ));
    commands.spawn((
        ChunkMember(cx, cy),
        Mesh2d(meshes.add(mesh_odd)),
        MeshMaterial2d(mat_odd),
        Transform::default(),
    ));

    let world_ox = cx * chunk_size;
    let world_oy = cy * chunk_size;

    let mut occupied: HashSet<(u32, u32)> = HashSet::new();

    for d in &chunk.deposits {
        if d.amount == 0 {
            continue;
        }
        occupied.insert((d.x, d.y));
        let wx = world_ox + d.x as i32;
        let wy = world_oy + d.y as i32;

        if let Some(handle) = textures.base.get(&d.resource) {
            commands.spawn((
                ChunkMember(cx, cy),
                ResourceDeposit {
                    resource: d.resource.clone(),
                    amount: d.amount,
                },
                Sprite {
                    image: handle.clone(),
                    custom_size: Some(Vec2::new(tile_size * 0.8, tile_size * 0.8)),
                    ..default()
                },
                Transform::from_xyz(wx as f32 * tile_size, wy as f32 * tile_size, 0.5),
                TilePosition { x: wx, y: wy },
            ));
        } else {
            let color = res_registry
                .get_opt(&d.resource)
                .map(|d| d.color)
                .unwrap_or(Color::srgb(0.5, 0.5, 0.5));
            let dep_color = materials.add(color);
            commands.spawn((
                ChunkMember(cx, cy),
                ResourceDeposit {
                    resource: d.resource.clone(),
                    amount: d.amount,
                },
                Mesh2d(shapes.circle.clone()),
                MeshMaterial2d(dep_color),
                Transform::from_xyz(wx as f32 * tile_size, wy as f32 * tile_size, 0.5),
                TilePosition { x: wx, y: wy },
            ));
        }
    }

    let mut rng = SimpleRng::new(chunk_hash);
    let deco_count = 4 + (rng.next() as usize % 5);
    let deco_kinds = [
        ("tree", Color::srgb(0.15, 0.45, 0.15)),
        ("rock", Color::srgb(0.4, 0.4, 0.4)),
    ];
    for _ in 0..deco_count {
        let dx = rng.next() % CHUNK_SIZE;
        let dy = rng.next() % CHUNK_SIZE;
        if occupied.contains(&(dx, dy)) {
            continue;
        }
        occupied.insert((dx, dy));
        let wx = world_ox + dx as i32;
        let wy = world_oy + dy as i32;
        let kind_idx = rng.next() as usize % deco_kinds.len();
        let (kind_name, color) = &deco_kinds[kind_idx];
        let mesh = if *kind_name == "tree" {
            shapes.triangle.clone()
        } else {
            shapes.circle.clone()
        };
        let z = if *kind_name == "tree" { 0.3 } else { 0.2 };
        let mat = materials.add(*color);
        commands.spawn((
            ChunkMember(cx, cy),
            Decoration(kind_name.to_string()),
            Mesh2d(mesh),
            MeshMaterial2d(mat),
            Transform::from_xyz(wx as f32 * tile_size, wy as f32 * tile_size, z),
            TilePosition { x: wx, y: wy },
        ));
    }
}

pub fn spawn_chunks_in_range(
    commands: &mut Commands,
    chunk_grid: &mut ChunkGrid,
    cfg: &MapConfig,
    res_registry: &ResourceRegistry,
    shapes: &ShapeCache,
    materials: &mut Assets<ColorMaterial>,
    meshes: &mut Assets<Mesh>,
    textures: &TextureCache,
    min_cx: i32,
    max_cx: i32,
    min_cy: i32,
    max_cy: i32,
    existing: &HashSet<(i32, i32)>,
) {
    for cx in min_cx..=max_cx {
        for cy in min_cy..=max_cy {
            if existing.contains(&(cx, cy)) {
                continue;
            }
            spawn_single_chunk_visuals(
                commands,
                chunk_grid,
                cfg,
                res_registry,
                shapes,
                materials,
                meshes,
                textures,
                cx,
                cy,
            );
        }
    }
}

pub fn update_visible_chunks(
    mut commands: Commands,
    camera: Query<(&Camera, &Transform)>,
    window: Query<&Window>,
    mut chunk_grid: ResMut<ChunkGrid>,
    cfg: Res<MapConfig>,
    res_registry: Res<ResourceRegistry>,
    existing_markers: Query<(Entity, &super::ChunkMarker)>,
    existing_members: Query<(Entity, &ChunkMember)>,
    existing_deposits: Query<(Entity, &ResourceDeposit, &TilePosition)>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    textures: Res<TextureCache>,
    _peaceful: Res<PeacefulMode>,
) {
    let Ok((cam, cam_transform)) = camera.single() else {
        return;
    };
    let Ok(window) = window.single() else { return };

    let global_tf = GlobalTransform::from(*cam_transform);
    let Some(top_left) = cam
        .viewport_to_world_2d(&global_tf, Vec2::new(0.0, 0.0))
        .ok()
    else {
        return;
    };
    let Some(bottom_right) = cam
        .viewport_to_world_2d(&global_tf, Vec2::new(window.width(), window.height()))
        .ok()
    else {
        return;
    };

    let tile_size = cfg.tile_size;
    let chunk_size = CHUNK_SIZE as i32;
    let visible_w_tiles = ((bottom_right.x - top_left.x) / tile_size).abs().ceil() as i32;
    let visible_h_tiles = ((bottom_right.y - top_left.y) / tile_size).abs().ceil() as i32;
    let margin = visible_w_tiles.max(visible_h_tiles).max(chunk_size * 2);

    let min_tx = (top_left.x / tile_size).floor() as i32 - margin;
    let max_tx = (bottom_right.x / tile_size).ceil() as i32 + margin;
    let min_ty = (top_left.y / tile_size).floor() as i32 - margin;
    let max_ty = (bottom_right.y / tile_size).ceil() as i32 + margin;

    let min_cx = min_tx.div_euclid(chunk_size);
    let max_cx = max_tx.div_euclid(chunk_size);
    let min_cy = min_ty.div_euclid(chunk_size);
    let max_cy = max_ty.div_euclid(chunk_size);

    let mut spawned: HashSet<(i32, i32)> = HashSet::new();
    for (_, marker) in existing_markers.iter() {
        spawned.insert((marker.0, marker.1));
    }

    let despawn_margin = 3;
    let mut deposit_updates: Vec<((i32, i32, u32, u32), u32)> = Vec::new();
    let mut to_despawn: Vec<Entity> = Vec::new();

    for (entity, marker) in existing_markers.iter() {
        let (cx, cy) = (marker.0, marker.1);
        if cx < min_cx - despawn_margin
            || cx > max_cx + despawn_margin
            || cy < min_cy - despawn_margin
            || cy > max_cy + despawn_margin
        {
            to_despawn.push(entity);
            let world_ox = cx * chunk_size;
            let world_oy = cy * chunk_size;
            for (_dep_entity, deposit, pos) in existing_deposits.iter() {
                if pos.x >= world_ox
                    && pos.x < world_ox + CHUNK_SIZE as i32
                    && pos.y >= world_oy
                    && pos.y < world_oy + CHUNK_SIZE as i32
                {
                    let dx = (pos.x - world_ox) as u32;
                    let dy = (pos.y - world_oy) as u32;
                    deposit_updates.push(((cx, cy, dx, dy), deposit.amount));
                }
            }
        }
    }

    for (entity, member) in existing_members.iter() {
        let (cx, cy) = (member.0, member.1);
        if cx < min_cx - despawn_margin
            || cx > max_cx + despawn_margin
            || cy < min_cy - despawn_margin
            || cy > max_cy + despawn_margin
        {
            to_despawn.push(entity);
        }
    }

    for entity in to_despawn {
        commands.entity(entity).despawn();
    }

    for ((cx, cy, dx, dy), amount) in deposit_updates {
        let chunk = chunk_grid.ensure_chunk_mut(cx, cy);
        for d in chunk.deposits.iter_mut() {
            if d.x == dx && d.y == dy {
                d.amount = amount;
                break;
            }
        }
    }

    spawn_chunks_in_range(
        &mut commands,
        &mut chunk_grid,
        &cfg,
        &res_registry,
        &shapes,
        &mut materials,
        &mut meshes,
        &textures,
        min_cx,
        max_cx,
        min_cy,
        max_cy,
        &spawned,
    );
}
