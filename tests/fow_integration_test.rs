use bevy::prelude::*;
use bevy::asset::AssetPlugin;
use bevy::MinimalPlugins;
use bevy::state::app::StatesPlugin;
use siege_factory::core::game_state::{GameState, IsFreshGame};
use siege_factory::economy::components::Player;
use siege_factory::economy::game_components::PeacefulMode;
use siege_factory::economy::resource::ResourceRegistry;
use siege_factory::economy::building::BuildingRegistry;
use siege_factory::economy::discovery::{DiscoveryRegistry, GlobalArchive};
use siege_factory::economy::menu::{MenuDef, MenuState, MenuItems};
use siege_factory::economy::unit_config::UnitConfig;
use siege_factory::map::components::{ChunkMember, FogTile, TilePosition};
use siege_factory::map::config::MapConfig;
use siege_factory::map::tile_grid::{CHUNK_SIZE, ChunkGrid};
use siege_factory::map::systems::{update_fog_of_war, ChunkMarker};
use siege_factory::rendering::config::VisualsConfig;
use siege_factory::rendering::cache::{ShapeCache, PreviewMaterials, TextureCache};
use siege_factory::economy::player::PlayerWorldPos;
use bevy::ecs::system::RunSystemOnce;
use siege_factory::core::modding::ModRegistry;


/// Helper to create test resources
fn test_mods() -> ModRegistry { ModRegistry::for_test() }
fn test_dist() -> Vec<(String, u32)> {
    vec![
        ("iron_ore".to_string(), 50),
        ("copper_ore".to_string(), 35),
        ("coal".to_string(), 15),
    ]
}

fn build_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(StatesPlugin);

    // Core game state
    app.init_state::<GameState>();
    app.insert_resource(NextState::<GameState>::default());
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.insert_resource(IsFreshGame(true));
    app.insert_resource(PeacefulMode(true));

    // Map resources
    let mut map_cfg = MapConfig::load(&test_mods());
    map_cfg.player_start_position = (10, 10);
    map_cfg.initial_margin = 2;
    app.insert_resource(map_cfg);

    let cfg = app.world().resource::<MapConfig>();
    let (seed, dep_min, dep_max, dep_chance, dep_min_per, dep_max_per) = (
        cfg.seed, cfg.deposit_min_amount, cfg.deposit_max_amount,
        cfg.deposit_spawn_chance_pct, cfg.deposit_min_per_chunk, cfg.deposit_max_per_chunk,
    );
    app.insert_resource(ChunkGrid::new(
        seed, dep_min, dep_max, dep_chance, dep_min_per, dep_max_per, test_dist(),
    ));

    // Load registries from TOML
    let discovery_registry = DiscoveryRegistry::load(&test_mods());
    app.insert_resource(GlobalArchive::new(&discovery_registry.starter_recipes));
    app.insert_resource(discovery_registry);
    app.insert_resource(ResourceRegistry::load(&test_mods()));
    app.init_resource::<PlayerWorldPos>();

    let building_registry = BuildingRegistry::load(&test_mods());
    let unit_cfg = UnitConfig::load(&test_mods());
    let menu_def = MenuDef::load(&test_mods(), &building_registry, &unit_cfg);
    app.insert_resource(building_registry);
    app.insert_resource(unit_cfg);
    app.insert_resource(menu_def);
    app.init_resource::<MenuState>();
    app.init_resource::<MenuItems>();

    // Load visuals
    app.insert_resource(VisualsConfig::load(&test_mods()));

    // Initialize asset storage needed by rendering caches
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<ColorMaterial>>();

    // Rendering caches (FromWorld — needs MapConfig + Assets in World first)
    app.init_resource::<ShapeCache>();
    app.init_resource::<PreviewMaterials>();
    app.insert_resource(TextureCache::default());

    // Add a Window entity so update_visible_chunks doesn't early-return
    app.world_mut().spawn(Window::default());

    // Add our test systems
    app.add_systems(Update, update_fog_of_war);

    // Spawn camera and player
    let cfg = app.world().resource::<MapConfig>().clone();
    let (bx, by) = cfg.player_start_position;
    let pos = siege_factory::core::utils::tile_to_world(bx, by, cfg.tile_size);

    app.world_mut().spawn((
        Camera2d,
        Camera::default(),
        Transform::from_xyz(pos.x, pos.y, 100.0),
        GlobalTransform::default(),
    ));

    app.world_mut().spawn((
        Player,
        TilePosition { x: bx, y: by },
        Transform::from_xyz(pos.x, pos.y, 5.0),
        GlobalTransform::default(),
    ));

    app
}

/// Spawn a chunk at (cx, cy) by pushing to pending_spawns and running
/// single_chunk_spawner once.
fn spawn_one_chunk(app: &mut App, cx: i32, cy: i32) {
    {
        let mut cg = app.world_mut().resource_mut::<ChunkGrid>();
        cg.pending_spawns.push((cx, cy));
    }
    let _ = app.world_mut().run_system_once(spawn_single_chunk);
}

fn spawn_single_chunk(
    mut commands: Commands,
    mut chunk_grid: ResMut<ChunkGrid>,
    cfg: Res<MapConfig>,
    res_registry: Res<ResourceRegistry>,
    global_archive: Res<GlobalArchive>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    textures: Res<TextureCache>,
    visuals: Res<VisualsConfig>,
    preview: Res<PreviewMaterials>,
) {
    if let Some(&(cx, cy)) = chunk_grid.pending_spawns.first() {
        siege_factory::map::systems::spawn_single_chunk_visuals(
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

#[test]
fn initial_chunks_are_spawned_over_multiple_frames() {
    let mut app = build_app();

    // Step: OnEnter(Playing) — setup_map pushes pending spawns
    let cfg = app.world().resource::<MapConfig>().clone();
    let chunk_size = CHUNK_SIZE as i32;
    let (px, py) = cfg.player_start_position;
    let player_cx = px.div_euclid(chunk_size);
    let player_cy = py.div_euclid(chunk_size);
    let margin = cfg.initial_margin;

    let total_pending: u32 = ((margin * 2 + 1) as u32).pow(2);

    for cx in player_cx - margin..=player_cx + margin {
        for cy in player_cy - margin..=player_cy + margin {
            spawn_one_chunk(&mut app, cx, cy);
        }
    }

    // Run FOW once to ensure fog tiles are interacted with
    app.update();

    // Verify chunks were actually spawned (ChunkMarker entities exist)
    let chunk_count = {
        let mut q = app.world_mut().query::<&ChunkMarker>();
        q.iter(&app.world_mut()).count()
    };
    assert_eq!(chunk_count, total_pending as usize, "All pending chunks should have been spawned");

    // Verify each chunk has a FogTile
    let fog_count = {
        let mut q = app.world_mut().query::<&FogTile>();
        q.iter(&app.world_mut()).count()
    };
    assert!(fog_count > 0, "Fog tiles should exist for spawned chunks");
    assert_eq!(
        fog_count, total_pending as usize,
        "Each chunk should have exactly one FogTile (no duplicates)"
    );
}

#[test]
fn fow_reveals_tiles_around_player() {
    let mut app = build_app();

    // Spawn chunks around the player
    let cfg = app.world().resource::<MapConfig>().clone();
    let chunk_size = CHUNK_SIZE as i32;
    let (px, py) = cfg.player_start_position;
    let player_cx = px.div_euclid(chunk_size);
    let player_cy = py.div_euclid(chunk_size);
    let margin = cfg.initial_margin;

    for cx in player_cx - margin..=player_cx + margin {
        for cy in player_cy - margin..=player_cy + margin {
            spawn_one_chunk(&mut app, cx, cy);
        }
    }

    // Run FOW so player tile gets marked as visited
    app.update();

    // Check that player's tile is marked as visited after FOW runs
    let chunk_grid = app.world().resource::<ChunkGrid>();
    let player_chunk_cx = px.div_euclid(CHUNK_SIZE as i32);
    let player_chunk_cy = py.div_euclid(CHUNK_SIZE as i32);
    let player_local_x = px.rem_euclid(CHUNK_SIZE as i32) as u32;
    let player_local_y = py.rem_euclid(CHUNK_SIZE as i32) as u32;

    assert!(
        chunk_grid.is_tile_visited(player_chunk_cx, player_chunk_cy, player_local_x, player_local_y),
        "Player's starting tile should be visited after FOW runs"
    );

    // Check that tiles in a 6-tile radius are visited
    let mut visited_any = false;
    let reveal_radius = 6i32;
    for dx in -reveal_radius..=reveal_radius {
        for dy in -reveal_radius..=reveal_radius {
            let wx = px + dx;
            let wy = py + dy;
            let cx = wx.div_euclid(CHUNK_SIZE as i32);
            let cy = wy.div_euclid(CHUNK_SIZE as i32);
            let tx = wx.rem_euclid(CHUNK_SIZE as i32) as u32;
            let ty = wy.rem_euclid(CHUNK_SIZE as i32) as u32;
            if chunk_grid.is_tile_visited(cx, cy, tx, ty) {
                visited_any = true;
                break;
            }
        }
        if visited_any { break; }
    }
    assert!(visited_any, "Tiles around player should be visited after FOW runs");
}

#[test]
fn fow_reveals_new_tiles_when_player_moves() {
    let mut app = build_app();

    // Spawn chunks around the player
    let cfg = app.world().resource::<MapConfig>().clone();
    let chunk_size = CHUNK_SIZE as i32;
    let (px, py) = cfg.player_start_position;
    let player_cx = px.div_euclid(chunk_size);
    let player_cy = py.div_euclid(chunk_size);
    let margin = cfg.initial_margin;

    for cx in player_cx - margin..=player_cx + margin {
        for cy in player_cy - margin..=player_cy + margin {
            spawn_one_chunk(&mut app, cx, cy);
        }
    }

    // Run FOW once to mark initial tiles
    app.update();

    // Move the player 10 tiles right
    let new_tile_x = px + 10;
    let new_tile_y = py;

    let mut player_query = app.world_mut().query::<&mut TilePosition>();
    for mut tp in player_query.iter_mut(&mut app.world_mut()) {
        tp.x = new_tile_x;
        tp.y = new_tile_y;
    }

    // Run FOW again with new position
    app.update();

    // Verify the new area around (new_tile_x, new_tile_y) is visited
    let chunk_grid = app.world().resource::<ChunkGrid>();
    let new_chunk_cx = new_tile_x.div_euclid(CHUNK_SIZE as i32);
    let new_chunk_cy = new_tile_y.div_euclid(CHUNK_SIZE as i32);
    let new_local_x = new_tile_x.rem_euclid(CHUNK_SIZE as i32) as u32;
    let new_local_y = new_tile_y.rem_euclid(CHUNK_SIZE as i32) as u32;

    assert!(
        chunk_grid.is_tile_visited(new_chunk_cx, new_chunk_cy, new_local_x, new_local_y),
        "Player's new position tile should be visited after movement"
    );

    // Check for duplicate FogTiles
    let fog_entities: Vec<(Entity, ChunkMember)> = {
        let mut q = app.world_mut().query::<(Entity, &ChunkMember)>();
        q.iter(&app.world_mut()).map(|(e, cm)| (e, cm.clone())).collect()
    };

    // Count fog entities per chunk
    let mut fog_per_chunk: std::collections::HashMap<(i32,i32), u32> = std::collections::HashMap::new();
    for (entity, cm) in &fog_entities {
        let has_fog = app.world().entity(*entity).contains::<FogTile>();
        if has_fog {
            *fog_per_chunk.entry((cm.0, cm.1)).or_default() += 1;
        }
    }

    let max_dup = fog_per_chunk.values().max().copied().unwrap_or(0);
    assert_eq!(max_dup, 1, "No chunk should have more than 1 FogTile (found {max_dup} max)");
}
