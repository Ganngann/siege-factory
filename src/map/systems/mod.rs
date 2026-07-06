pub mod chunks;

pub use chunks::{
    build_chunk_mesh, spawn_chunks_in_range, spawn_single_chunk_visuals, update_visible_chunks,
};

use crate::core::game_state::GameState;
use crate::core::utils::tile_to_world;
use crate::economy::components::UiIsBlocking;
use crate::map::components::{ChunkMember, HoveredTile, cursor_to_tile};
use crate::map::config::MapConfig;
use crate::map::tile_grid::{CHUNK_SIZE, ChunkGrid};
use bevy::prelude::{
    App, Assets, ButtonInput, Camera, Camera2d, ColorMaterial, Commands, Component, Entity,
    GlobalTransform, IntoScheduleConfigs, KeyCode, Mesh, OnEnter, OnExit, Plugin, Query, Res,
    ResMut, Transform, Update, Window, With, in_state,
};

#[derive(Component)]
pub struct ChunkMarker(pub i32, pub i32);

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        let cfg = MapConfig::load();
        let seed = cfg.seed;
        let dep_min = cfg.deposit_min_amount;
        let dep_max = cfg.deposit_max_amount;
        let dep_chance = cfg.deposit_spawn_chance_pct;
        let dep_min_per = cfg.deposit_min_per_chunk;
        let dep_max_per = cfg.deposit_max_per_chunk;
        let dep_dist = cfg.deposit_distribution.clone();
        app.insert_resource(cfg);
        app.insert_resource(ChunkGrid::new(
            seed,
            dep_min,
            dep_max,
            dep_chance,
            dep_min_per,
            dep_max_per,
            dep_dist,
        ));
        app.insert_resource(HoveredTile::default());
        app.add_systems(
            OnEnter(GameState::Playing),
            setup_map.run_if(crate::save_load::is_fresh_game),
        );
        app.add_systems(OnExit(GameState::Playing), cleanup_map);
        app.add_systems(
            Update,
            update_hovered_tile.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            update_visible_chunks.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            recenter_on_player.run_if(in_state(GameState::Playing)),
        );
    }
}

fn setup_map(
    mut commands: Commands,
    cfg: Res<MapConfig>,
    mut chunk_grid: ResMut<ChunkGrid>,
    res_registry: Res<crate::economy::resource::ResourceRegistry>,
    shapes: Res<crate::rendering::ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    textures: Res<crate::rendering::TextureCache>,
    visuals: Res<crate::rendering::config::VisualsConfig>,
) {
    let (px, py) = cfg.player_start_position;
    let chunk_size = CHUNK_SIZE as i32;
    let margin_chunks = cfg.initial_margin;
    let player_cx = px.div_euclid(chunk_size);
    let player_cy = py.div_euclid(chunk_size);
    let existing = std::collections::HashSet::new();
    spawn_chunks_in_range(
        &mut commands,
        &mut chunk_grid,
        &cfg,
        &res_registry,
        &shapes,
        &mut materials,
        &mut meshes,
        &textures,
        &visuals,
        player_cx - margin_chunks,
        player_cx + margin_chunks,
        player_cy - margin_chunks,
        player_cy + margin_chunks,
        &existing,
    );
}

fn recenter_on_player(
    keys: Res<ButtonInput<KeyCode>>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    cfg: Res<MapConfig>,
) {
    if !keys.just_pressed(KeyCode::KeyH) {
        return;
    }
    let (px, py) = cfg.player_start_position;
    if let Ok(mut tf) = camera.single_mut() {
        let pos = tile_to_world(px, py, cfg.tile_size);
        tf.translation.x = pos.x;
        tf.translation.y = pos.y;
    }
}

fn update_hovered_tile(
    mut hovered: ResMut<HoveredTile>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    cfg: Res<MapConfig>,
    ui_blocking: Res<UiIsBlocking>,
) {
    if ui_blocking.0 {
        hovered.0 = None;
        return;
    }
    hovered.0 = cursor_to_tile(&windows, &camera, &cfg);
}

fn cleanup_map(
    mut commands: Commands,
    markers: Query<Entity, With<ChunkMarker>>,
    members: Query<Entity, With<ChunkMember>>,
    cameras: Query<Entity, With<Camera2d>>,
    mut chunk_grid: ResMut<ChunkGrid>,
) {
    for entity in markers.iter() {
        commands.entity(entity).despawn();
    }
    for entity in members.iter() {
        commands.entity(entity).despawn();
    }
    for entity in cameras.iter() {
        commands.entity(entity).despawn();
    }
    chunk_grid.clear();
}
