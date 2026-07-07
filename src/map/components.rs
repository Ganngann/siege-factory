use crate::core::utils::world_to_tile;
use crate::map::config::MapConfig;
use crate::rendering::minimap::MinimapCamera;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TileType {
    #[default]
    Ground,
    Resource,
}

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq, Hash)]
pub struct TilePosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq, Hash)]
pub struct ChunkMember(pub i32, pub i32);

#[derive(Component)]
pub struct Decoration(pub String);

#[derive(Resource, Default)]
pub struct HoveredTile(pub Option<TilePosition>);

#[derive(Component)]
pub struct HiddenDeposit {
    pub required_discovery: String,
}

#[derive(Component)]
pub struct FogTile;

pub fn cursor_to_tile(
    windows: &Query<&Window>,
    camera: &Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<MinimapCamera>)>,
    cfg: &MapConfig,
) -> Option<TilePosition> {
    let window = windows.single().ok()?;
    let cursor = window.cursor_position()?;
    let (cam, cam_tf) = camera.single().ok()?;
    let world_pos = cam.viewport_to_world_2d(cam_tf, cursor).ok()?;
    let (tx, ty) = world_to_tile(world_pos, cfg.tile_size);
    Some(TilePosition { x: tx, y: ty })
}
