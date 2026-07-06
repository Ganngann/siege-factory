use crate::economy::resource::ResourceId;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct HQ;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Builder {
    pub state: BuilderState,
}

#[derive(Clone)]
pub enum BuilderState {
    Idle,
    MovingToBuilding(Entity),
    ReturningToPlayer,
}

#[derive(Component)]
pub struct UnbuiltBuilding;

#[derive(Component)]
pub struct ResourceDeposit {
    pub resource: String,
    pub amount: u32,
}

#[derive(Component)]
pub struct Miner;

#[derive(Component, Clone)]
pub struct Assembler {
    pub production_timer: f32,
    pub interval: f32,
    pub recipe_id: String,
}

#[derive(Component)]
pub struct Building {
    pub kind: String,
    pub name: String,
}

#[derive(Component)]
pub struct Ghost;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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

    pub fn prev(&self) -> Self {
        match self {
            Direction::East => Direction::South,
            Direction::North => Direction::East,
            Direction::West => Direction::North,
            Direction::South => Direction::West,
        }
    }

    pub fn from_offset(dx: i32, dy: i32) -> Self {
        match (dx, dy) {
            (1, 0) => Direction::East,
            (0, 1) => Direction::North,
            (-1, 0) => Direction::West,
            (0, -1) => Direction::South,
            _ => Direction::East,
        }
    }

    pub fn perpendicular(&self) -> [Direction; 2] {
        match self {
            Direction::East | Direction::West => [Direction::North, Direction::South],
            Direction::North | Direction::South => [Direction::East, Direction::West],
        }
    }
}

#[derive(Component, Clone)]
pub struct TurretCombat {
    pub damage: u32,
    pub range_sq: f32,
    pub fire_interval: f32,
    pub timer: f32,
    pub projectile_speed: f32,
}

#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct HpBarChild;

#[derive(Component)]
pub struct HasHpBar;

#[derive(Component)]
pub struct OccupiedTiles(pub Vec<(i32, i32)>);

#[derive(Component)]
pub struct Storage;

#[derive(Component)]
pub struct Splitter {
    pub counter: u32,
    pub outputs: u32,
    pub input_direction: Option<Direction>,
}

#[derive(Component, Clone)]
pub struct Sorter {
    pub filter: ResourceId,
    pub inverted: bool,
}

#[derive(Component)]
pub struct Active(pub bool);

#[derive(Resource, Default)]
pub struct PeacefulMode(pub bool);
