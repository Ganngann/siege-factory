use bevy::prelude::*;

#[derive(Component)]
pub struct HQ;

#[derive(Component)]
pub struct OreDeposit {
    pub amount: u32,
}

#[derive(Component)]
pub struct Miner {
    pub production_timer: f32,
    pub interval: f32,
}

#[derive(Component)]
pub struct Assembler {
    pub production_timer: f32,
    pub interval: f32,
}

#[derive(Component)]
pub struct Building {
    pub name: String,
}

#[derive(Component)]
pub struct Turret {
    pub fire_timer: f32,
}

#[derive(Component)]
pub struct Ghost;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

    pub fn color(&self) -> Color {
        match self {
            Direction::East => Color::srgb(0.6, 0.5, 0.4),
            Direction::North => Color::srgb(0.5, 0.6, 0.4),
            Direction::West => Color::srgb(0.4, 0.5, 0.6),
            Direction::South => Color::srgb(0.5, 0.4, 0.6),
        }
    }
}

#[derive(Resource, Default)]
pub struct BuildMode(pub Option<BuildKind>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildKind {
    Miner,
    Assembler,
    Belt,
    Wall,
    Turret,
}

#[derive(Resource, Default)]
pub struct BeltDirection(pub Direction);

#[derive(Resource, Default)]
pub struct BuildPreview(pub Option<Entity>);

#[derive(Event)]
pub struct SetBuildModeEvent(pub Option<BuildKind>);
