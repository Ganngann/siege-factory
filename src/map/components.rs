use bevy::prelude::*;
use crate::map::config::MapConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TileType {
    #[default]
    Ground,
    Resource,
    Spawner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tile {
    pub tile_type: TileType,
    pub occupied: bool,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct TilePosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Resource, Default)]
pub struct HoveredTile(pub Option<TilePosition>);

pub fn cursor_to_tile(
    windows: &Query<&Window>,
    camera: &Query<(&Camera, &GlobalTransform)>,
    cfg: &MapConfig,
) -> Option<TilePosition> {
    let window = windows.single().ok()?;
    let cursor = window.cursor_position()?;
    let (cam, cam_transform) = camera.single().ok()?;
    let world_pos = cam.viewport_to_world_2d(cam_transform, cursor).ok()?;
    let tile_x = ((world_pos.x + cfg.tile_size / 2.0) / cfg.tile_size).floor() as i32;
    let tile_y = ((world_pos.y + cfg.tile_size / 2.0) / cfg.tile_size).floor() as i32;
    if tile_x < 0 || tile_y < 0 || tile_x >= cfg.width as i32 || tile_y >= cfg.height as i32 {
        return None;
    }
    Some(TilePosition {
        x: tile_x as u32,
        y: tile_y as u32,
    })
}
