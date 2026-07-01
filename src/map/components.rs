use bevy::prelude::*;

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
