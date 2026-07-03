use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::economy::resource::ResourceId;

#[derive(Component)]
pub struct HQ;

#[derive(Component)]
pub struct ResourceDeposit {
    pub resource: String,
    pub amount: u32,
}

#[derive(Component)]
pub struct Miner {
    pub production_timer: f32,
    pub interval: f32,
}

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

    pub fn color(&self) -> Color {
        match self {
            Direction::East => Color::srgb(0.6, 0.5, 0.4),
            Direction::North => Color::srgb(0.5, 0.6, 0.4),
            Direction::West => Color::srgb(0.4, 0.5, 0.6),
            Direction::South => Color::srgb(0.5, 0.4, 0.6),
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

#[derive(Resource, Default)]
pub struct BuildMode(pub Option<String>);

#[derive(Resource, Default)]
pub struct BeltDirection(pub Direction);

#[derive(Resource, Default)]
pub struct BuildPreview(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct BeltDrag {
    pub start_coord: Option<(i32, i32)>,
}

#[derive(Resource, Default)]
pub struct DeconstructMode(pub bool);

#[derive(Resource, Default)]
pub struct DeconstructDrag {
    pub start_coord: Option<(i32, i32)>,
}

#[derive(Resource)]
pub struct BuildingPopup {
    pub popup_entity: Option<Entity>,
    pub text_entity: Option<Entity>,
    pub inspected_entity: Option<Entity>,
    pub update_timer: f32,
    pub dirty: bool,
}

impl Default for BuildingPopup {
    fn default() -> Self {
        Self { popup_entity: None, text_entity: None, inspected_entity: None, update_timer: 0.0, dirty: false }
    }
}

// ── Generic behavior components ──

#[derive(Component)]
pub struct Produces {
    pub resource: ResourceId,
    pub interval: f32,
    pub timer: f32,
}

#[derive(Component, Clone)]
pub struct TurretCombat {
    pub damage: u32,
    pub range_sq: f32,
    pub fire_interval: f32,
    pub timer: f32,
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

#[derive(Resource, Default)]
pub struct PeacefulMode(pub bool);

#[derive(Event)]
pub struct SetBuildModeEvent(pub Option<String>);

// ── Menu UI components ──

#[derive(Component)]
pub struct MenuBarPanel;

#[derive(Component)]
pub struct BreadcrumbText;

#[derive(Component)]
pub struct BackButton;

#[derive(Component)]
pub struct ScrollButton(pub i32);

#[derive(Component)]
pub struct MenuItemButton {
    pub index: usize,
}

// ── Building popup UI ──

#[derive(Component)]
pub struct BuildingPopupRoot;

#[derive(Component)]
pub struct RecipeButton {
    pub recipe_id: String,
}

#[derive(Component)]
pub struct SorterResourceButton {
    pub resource: ResourceId,
}

#[derive(Component)]
pub struct CloseButton;

#[derive(Component)]
pub struct SorterInvertButton;

