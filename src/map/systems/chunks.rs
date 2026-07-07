use crate::core::utils::tile_to_world_corner;
use crate::economy::components::ResourceDeposit;
use crate::economy::discovery::GlobalArchive;
use crate::economy::resource::ResourceRegistry;
use crate::map::components::{ChunkMember, Decoration, FogTile, HiddenDeposit, TilePosition};
use crate::map::config::MapConfig;
use crate::map::rng::{SimpleRng, chunk_hash};
use crate::map::tile_grid::ChunkGrid;
use crate::map::tile_grid::{CHUNK_SIZE, Chunk};
use crate::rendering::config::VisualsConfig;
use crate::rendering::minimap::MinimapCamera;
use crate::rendering::{ShapeCache, TextureCache};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::{
    Assets, Camera, ColorMaterial, Commands, Entity, GlobalTransform, Mesh, Mesh2d, MeshMaterial2d,
    Query, Res, ResMut, Sprite, Transform, Vec2, Visibility, Window, With, Without, default,
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

fn build_fog_mesh(cx: i32, cy: i32, chunk: &Chunk, tile_size: f32) -> Mesh {
    let chunk_size = CHUNK_SIZE as i32;
    let world_ox = cx * chunk_size;
    let world_oy = cy * chunk_size;

    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut n = 0u32;

    for ty in 0..CHUNK_SIZE as usize {
        for tx in 0..CHUNK_SIZE as usize {
            if chunk.visited.contains(&(tx as u32, ty as u32)) {
                continue;
            }
            let wx = world_ox + tx as i32;
            let wy = world_oy + ty as i32;
            let pos = tile_to_world_corner(wx, wy, tile_size);
            let (x, y) = (pos.x, pos.y);
            let s = tile_size;

            positions.extend_from_slice(&[
                [x, y, 0.0],
                [x + s, y, 0.0],
                [x + s, y + s, 0.0],
                [x, y + s, 0.0],
            ]);
            indices.extend_from_slice(&[n, n + 1, n + 2, n, n + 2, n + 3]);
            n += 4;
        }
    }

    mesh_from_quads(positions, indices)
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
    global_archive: &GlobalArchive,
    shapes: &ShapeCache,
    materials: &mut Assets<ColorMaterial>,
    meshes: &mut Assets<Mesh>,
    textures: &TextureCache,
    visuals: &VisualsConfig,
    preview: &crate::rendering::cache::PreviewMaterials,
    cx: i32,
    cy: i32,
) {
    let chunk_size = CHUNK_SIZE as i32;
    let tile_size = cfg.tile_size;

    let chunk_hash = chunk_hash(cfg.seed, cx, cy);

    let (handle_even, handle_odd) = if let Some(cached) = chunk_grid.chunk_mesh_cache.get(&(cx, cy))
    {
        (cached.0.clone(), cached.1.clone())
    } else {
        let (mesh_even, mesh_odd) = build_chunk_mesh(cx, cy, tile_size);
        let he = meshes.add(mesh_even);
        let ho = meshes.add(mesh_odd);
        chunk_grid
            .chunk_mesh_cache
            .insert((cx, cy), (he.clone(), ho.clone()));
        (he, ho)
    };

    let chunk = chunk_grid.ensure_chunk(cx, cy);

    let mat_even = materials.add(visuals.chunk_colors.even);
    let mat_odd = materials.add(visuals.chunk_colors.odd);

    commands.spawn(super::ChunkMarker(cx, cy));
    commands.spawn((
        ChunkMember(cx, cy),
        Mesh2d(handle_even),
        MeshMaterial2d(mat_even),
        Transform::default(),
    ));
    commands.spawn((
        ChunkMember(cx, cy),
        Mesh2d(handle_odd),
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

        // Check if this resource requires discovery
        let is_hidden = cfg
            .resource_discovery_map
            .get(&d.resource)
            .map(|req| !global_archive.is_unlocked(req))
            .unwrap_or(false);

        if let Some(handle) = textures.base.get(&d.resource) {
            let mut entity_cmd = commands.spawn((
                ChunkMember(cx, cy),
                ResourceDeposit {
                    resource: d.resource.clone(),
                    amount: d.amount,
                },
                Sprite {
                    image: handle.clone(),
                    custom_size: Some(Vec2::new(
                        tile_size * visuals.deposit_sprite.scale_ratio,
                        tile_size * visuals.deposit_sprite.scale_ratio,
                    )),
                    ..default()
                },
                Transform::from_xyz(
                    wx as f32 * tile_size,
                    wy as f32 * tile_size,
                    visuals.deposit_sprite.z,
                ),
                TilePosition { x: wx, y: wy },
            ));
            if is_hidden {
                entity_cmd.insert(Visibility::Hidden);
                entity_cmd.insert(HiddenDeposit {
                    required_discovery: cfg.resource_discovery_map[&d.resource].clone(),
                });
            }
        } else {
            let color = res_registry
                .get_opt(&d.resource)
                .map(|d| d.color)
                .unwrap_or(visuals.deposit_sprite.fallback_color);
            let dep_color = materials.add(color);
            let mut entity_cmd = commands.spawn((
                ChunkMember(cx, cy),
                ResourceDeposit {
                    resource: d.resource.clone(),
                    amount: d.amount,
                },
                Mesh2d(shapes.circle.clone()),
                MeshMaterial2d(dep_color),
                Transform::from_xyz(
                    wx as f32 * tile_size,
                    wy as f32 * tile_size,
                    visuals.deposit_sprite.z,
                ),
                TilePosition { x: wx, y: wy },
            ));
            if is_hidden {
                entity_cmd.insert(Visibility::Hidden);
                entity_cmd.insert(HiddenDeposit {
                    required_discovery: cfg.resource_discovery_map[&d.resource].clone(),
                });
            }
        }
    }

    let mut rng = SimpleRng::new(chunk_hash);
    for deco in &visuals.decorations {
        let count = (deco.density * (CHUNK_SIZE * CHUNK_SIZE) as f32) as u32;
        let base = cfg.decoration_min_count + (rng.next() % (cfg.decoration_count_variance + 1));
        let total = count.max(base);
        for _ in 0..total {
            let dx = rng.next() % CHUNK_SIZE;
            let dy = rng.next() % CHUNK_SIZE;
            if occupied.contains(&(dx, dy)) {
                continue;
            }
            occupied.insert((dx, dy));
            let wx = world_ox + dx as i32;
            let wy = world_oy + dy as i32;
            let mesh = shapes.get_visual(&deco.shape);
            let mat = materials.add(deco.color);
            commands.spawn((
                ChunkMember(cx, cy),
                Decoration(deco.kind.clone()),
                Mesh2d(mesh),
                MeshMaterial2d(mat),
                Transform::from_xyz(wx as f32 * tile_size, wy as f32 * tile_size, deco.z),
                TilePosition { x: wx, y: wy },
            ));
        }
    }

    // Single fog mesh per chunk at z=1.0 (above deposits/decorations)
    let fog_mesh = build_fog_mesh(cx, cy, chunk, cfg.tile_size);
    commands.spawn((
        FogTile,
        Mesh2d(meshes.add(fog_mesh)),
        MeshMaterial2d(preview.fog.clone()),
        Transform::from_xyz(0.0, 0.0, 1.0),
        Visibility::default(),
        ChunkMember(cx, cy),
    ));
}

pub fn spawn_chunks_in_range(
    commands: &mut Commands,
    chunk_grid: &mut ChunkGrid,
    cfg: &MapConfig,
    res_registry: &ResourceRegistry,
    global_archive: &GlobalArchive,
    shapes: &ShapeCache,
    materials: &mut Assets<ColorMaterial>,
    meshes: &mut Assets<Mesh>,
    textures: &TextureCache,
    visuals: &VisualsConfig,
    preview: &crate::rendering::cache::PreviewMaterials,
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
                global_archive,
                shapes,
                materials,
                meshes,
                textures,
                visuals,
                preview,
                cx,
                cy,
            );
        }
    }
}

pub fn update_visible_chunks(
    mut commands: Commands,
    camera: Query<(&Camera, &Transform), Without<MinimapCamera>>,
    window: Query<&Window>,
    mut chunk_grid: ResMut<ChunkGrid>,
    cfg: Res<MapConfig>,
    res_registry: Res<ResourceRegistry>,
    global_archive: Res<GlobalArchive>,
    existing_markers: Query<(Entity, &super::ChunkMarker)>,
    existing_members: Query<(Entity, &ChunkMember)>,
    existing_deposits: Query<(Entity, &ResourceDeposit, &TilePosition)>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    textures: Res<TextureCache>,
    visuals: Res<VisualsConfig>,
    preview: Res<crate::rendering::cache::PreviewMaterials>,
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
    let margin = visible_w_tiles.max(visible_h_tiles).max(chunk_size);

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

    let despawn_margin = cfg.despawn_margin;
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

    chunk_grid.pending_spawns.retain(|&(cx, cy)| {
        cx >= min_cx - despawn_margin
            && cx <= max_cx + despawn_margin
            && cy >= min_cy - despawn_margin
            && cy <= max_cy + despawn_margin
            && !spawned.contains(&(cx, cy))
    });

    for cx in min_cx..=max_cx {
        for cy in min_cy..=max_cy {
            if !spawned.contains(&(cx, cy)) && !chunk_grid.pending_spawns.contains(&(cx, cy)) {
                chunk_grid.pending_spawns.push((cx, cy));
            }
        }
    }

    if let Some(&(cx, cy)) = chunk_grid.pending_spawns.first() {
        spawn_single_chunk_visuals(
            &mut commands,
            &mut chunk_grid,
            &cfg,
            &res_registry,
            &global_archive,
            &shapes,
            &mut materials,
            &mut meshes,
            &textures,
            &visuals,
            &preview,
            cx,
            cy,
        );
        chunk_grid.pending_spawns.remove(0);
    }
}

pub fn reveal_hidden_deposits(
    archive: Res<GlobalArchive>,
    mut hidden_deposits: Query<(&HiddenDeposit, &mut Visibility)>,
) {
    for (deposit, mut visibility) in hidden_deposits.iter_mut() {
        if archive.is_unlocked(&deposit.required_discovery) {
            *visibility = Visibility::Visible;
        }
    }
}

pub fn update_fog_of_war(
    player: Query<&TilePosition, With<crate::economy::game_components::Player>>,
    mut chunk_grid: ResMut<ChunkGrid>,
    mut commands: Commands,
    fog_tiles: Query<(Entity, &ChunkMember), With<FogTile>>,
    mut meshes: ResMut<Assets<Mesh>>,
    cfg: Res<MapConfig>,
) {
    let Ok(player_tile) = player.single() else {
        return;
    };

    let reveal_radius = 6i32;
    let (min_x, max_x) = (player_tile.x - reveal_radius, player_tile.x + reveal_radius);
    let (min_y, max_y) = (player_tile.y - reveal_radius, player_tile.y + reveal_radius);

    let mut affected_chunks: HashSet<(i32, i32)> = HashSet::new();

    for wx in min_x..=max_x {
        for wy in min_y..=max_y {
            let cx = wx.div_euclid(CHUNK_SIZE as i32);
            let cy = wy.div_euclid(CHUNK_SIZE as i32);
            let tx = wx.rem_euclid(CHUNK_SIZE as i32) as u32;
            let ty = wy.rem_euclid(CHUNK_SIZE as i32) as u32;

            if !chunk_grid.is_tile_visited(cx, cy, tx, ty) {
                affected_chunks.insert((cx, cy));
            }
            chunk_grid.reveal_tile(cx, cy, tx, ty);
        }
    }

    if affected_chunks.is_empty() {
        return;
    }

    // Build lookup of fog entity per chunk
    let fog_map: std::collections::HashMap<(i32, i32), Entity> = fog_tiles
        .iter()
        .filter(|(_, cm)| affected_chunks.contains(&(cm.0, cm.1)))
        .map(|(e, cm)| ((cm.0, cm.1), e))
        .collect();

    for &(cx, cy) in &affected_chunks {
        let total_tiles = (CHUNK_SIZE * CHUNK_SIZE) as usize;
        let visited_count = chunk_grid
            .get_chunk(cx, cy)
            .map(|c| c.visited.len())
            .unwrap_or(0);
        let all_visited = visited_count >= total_tiles;

        if all_visited {
            if let Some(&entity) = fog_map.get(&(cx, cy)) {
                commands.entity(entity).despawn();
            }
            continue;
        }

        if let Some(chunk) = chunk_grid.get_chunk(cx, cy) {
            let new_mesh = build_fog_mesh(cx, cy, chunk, cfg.tile_size);
            let handle = meshes.add(new_mesh);
            if let Some(&entity) = fog_map.get(&(cx, cy)) {
                commands.entity(entity).insert(Mesh2d(handle));
            }
        }
    }
}

pub fn apply_starting_area(
    mut commands: Commands,
    cfg: Res<MapConfig>,
    mut chunk_grid: ResMut<ChunkGrid>,
    visuals: Res<VisualsConfig>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let sa = &cfg.starting_area;
    if !sa.enable {
        return;
    }
    let radius = sa.radius as i32;
    let (px, py) = cfg.player_start_position;
    let tile_size = cfg.tile_size;
    let chunk_size = CHUNK_SIZE as i32;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx * dx + dy * dy > radius * radius {
                continue;
            }
            let wx = px + dx;
            let wy = py + dy;
            let cx = wx.div_euclid(chunk_size);
            let cy = wy.div_euclid(chunk_size);
            let local_x = wx.rem_euclid(chunk_size) as u32;
            let local_y = wy.rem_euclid(chunk_size) as u32;
            let chunk = chunk_grid.ensure_chunk_mut(cx, cy);

            if sa.clear_trees {
                chunk
                    .deposits
                    .retain(|d| !(d.x == local_x && d.y == local_y));
            }
        }
    }

    for structure in &sa.structures {
        let wx = structure.tile_x;
        let wy = structure.tile_y;
        let cx = wx.div_euclid(chunk_size);
        let cy = wy.div_euclid(chunk_size);
        let local_x = wx.rem_euclid(chunk_size) as u32;
        let local_y = wy.rem_euclid(chunk_size) as u32;

        match structure.kind.as_str() {
            "deposit" => {
                if let (Some(resource), Some(amount)) =
                    (&structure.props.resource, structure.props.amount)
                {
                    let chunk = chunk_grid.ensure_chunk_mut(cx, cy);
                    chunk.tiles[local_y as usize][local_x as usize] =
                        crate::map::components::TileType::Resource;
                    chunk.deposits.push(crate::map::tile_grid::Deposit {
                        x: local_x,
                        y: local_y,
                        amount,
                        resource: resource.clone(),
                    });
                }
            }
            "decoration" => {
                if let Some(deco_kind) = &structure.props.decoration_kind {
                    if let Some(deco_cfg) =
                        visuals.decorations.iter().find(|d| d.kind == *deco_kind)
                    {
                        let mesh = shapes.get_visual(&deco_cfg.shape);
                        let mat = materials.add(deco_cfg.color);
                        commands.spawn((
                            ChunkMember(cx, cy),
                            Decoration(deco_kind.clone()),
                            Mesh2d(mesh),
                            MeshMaterial2d(mat),
                            Transform::from_xyz(
                                wx as f32 * tile_size,
                                wy as f32 * tile_size,
                                deco_cfg.z,
                            ),
                            TilePosition { x: wx, y: wy },
                        ));
                    }
                }
            }
            _ => {}
        }
    }
}
