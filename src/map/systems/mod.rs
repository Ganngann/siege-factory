pub mod chunks;

pub use chunks::{
    apply_starting_area, build_chunk_mesh, reveal_hidden_deposits, spawn_chunks_in_range,
    spawn_single_chunk_visuals, update_fog_of_war, update_visible_chunks,
};

use crate::core::game_state::GameState;
use crate::core::schedule::GameplayStep;
use crate::core::utils::{silent_despawn, tile_to_world};
use crate::economy::components::UiIsBlocking;
use crate::map::components::{ChunkMember, HoveredTile, cursor_to_tile};
use crate::map::config::MapConfig;
use crate::map::tile_grid::{CHUNK_SIZE, ChunkGrid};
use crate::rendering::minimap::MinimapCamera;
use bevy::prelude::{
    App, ButtonInput, Camera, Camera2d, Commands, Component, Entity, GlobalTransform,
    IntoScheduleConfigs, KeyCode, OnEnter, OnExit, Plugin, Query, Res, ResMut, Transform, Update,
    Window, With, Without, in_state,
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
            (setup_map, apply_starting_area)
                .chain()
                .run_if(crate::save_load::is_fresh_game),
        );
        app.add_systems(OnExit(GameState::Playing), cleanup_map);
        app.add_systems(
            Update,
            update_hovered_tile.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            update_visible_chunks
                .run_if(in_state(GameState::Playing))
                .in_set(GameplayStep::ChunkManagement),
        );
        app.add_systems(
            Update,
            reveal_hidden_deposits.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            update_fog_of_war
                .run_if(in_state(GameState::Playing))
                .in_set(GameplayStep::FogOfWar),
        );
        app.add_systems(
            Update,
            recenter_on_player.run_if(in_state(GameState::Playing)),
        );
    }
}

fn setup_map(
    mut chunk_grid: ResMut<ChunkGrid>,
    cfg: Res<MapConfig>,
) {
    let chunk_size = CHUNK_SIZE as i32;
    let (px, py) = cfg.player_start_position;
    let player_cx = px.div_euclid(chunk_size);
    let player_cy = py.div_euclid(chunk_size);
    let margin = cfg.initial_margin;
    for cx in player_cx - margin..=player_cx + margin {
        for cy in player_cy - margin..=player_cy + margin {
            chunk_grid.pending_spawns.push((cx, cy));
        }
    }
}

pub fn recenter_on_player(
    keys: Res<ButtonInput<KeyCode>>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<MinimapCamera>)>,
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

pub fn update_hovered_tile(
    mut hovered: ResMut<HoveredTile>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<MinimapCamera>)>,
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
        silent_despawn(&mut commands, entity);
    }
    for entity in members.iter() {
        silent_despawn(&mut commands, entity);
    }
    for entity in cameras.iter() {
        silent_despawn(&mut commands, entity);
    }
    chunk_grid.clear();
}
