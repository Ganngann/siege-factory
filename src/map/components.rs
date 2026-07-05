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

pub fn cursor_to_tile(
    windows: &Query<&Window>,
    camera: &Query<(&Camera, &Transform)>,
    tile_size: f32,
) -> Option<TilePosition> {
    let window = windows.single().ok()?;
    let cursor = window.cursor_position()?;
    let (cam, cam_tf) = camera.single().ok()?;
    let world_pos = cam.viewport_to_world_2d(&GlobalTransform::from(*cam_tf), cursor).ok()?;
    let tile_x = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let tile_y = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;
    Some(TilePosition { x: tile_x, y: tile_y })
}
