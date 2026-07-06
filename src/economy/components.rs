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

// ── Inventory UI components ──

#[derive(Component)]
pub struct InventoryGrid {
    pub cols: usize,
    pub rows: usize,
    pub owner: Entity,
}

#[derive(Component)]
pub struct InventorySlot {
    pub index: usize,
}

#[derive(Component)]
pub struct DraggedItemVisual;

#[derive(Resource, Default)]
pub struct DragState {
    pub active: bool,
    pub source_slot: Option<Entity>,
    pub source_owner: Option<Entity>,
    pub resource: Option<crate::economy::resource::ResourceId>,
    pub amount: u32,
    pub visual: Option<Entity>,
}

impl DragState {
    pub fn reset(&mut self) {
        self.active = false;
        self.source_slot = None;
        self.source_owner = None;
        self.resource = None;
        self.amount = 0;
        self.visual = None;
    }
}

// ── End inventory UI components ──

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

#[derive(Resource, Default)]
pub struct UiIsBlocking(pub bool);

#[derive(Resource)]
pub struct BuildingPanel {
    pub inspected: Option<Entity>,
    pub root: Option<Entity>,
    pub overlay: Option<Entity>,
    pub recipe_selector: Option<Entity>,
    pub dirty: bool,
}

impl Default for BuildingPanel {
    fn default() -> Self {
        Self {
            inspected: None,
            root: None,
            overlay: None,
            recipe_selector: None,
            dirty: false,
        }
    }
}

// ── Generic behavior components ──

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

// ── Electricity ──

#[derive(Component)]
pub struct PowerConsumer {
    pub draw: f32,
    pub satisfied: bool,
}

#[derive(Component)]
pub struct PowerProducer {
    pub output: f32,
}

#[derive(Component)]
pub struct PowerPole {
    pub range: f32,
}

#[derive(Resource, Default)]
pub struct PeacefulMode(pub bool);

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

// ── Building panel UI ──

#[derive(Component)]
pub struct PanelOverlay;
#[derive(Component)]
pub struct PanelModal;

#[derive(Component)]
pub struct BuildingTitleText;
#[derive(Component)]
pub struct DragHandle;
#[derive(Component)]
pub struct ActiveToggleButton;
#[derive(Component)]
pub struct CloseButton;
#[derive(Component)]
pub struct ProgressBarBg;
#[derive(Component)]
pub struct ProgressBarFill;
#[derive(Component)]
pub struct StatusText;
#[derive(Component)]
pub struct FlowInputText;
#[derive(Component)]
pub struct FlowOutputText;
#[derive(Component)]
pub struct CapacityBarFill;
#[derive(Component)]
pub struct CapacityBarText;
#[derive(Component)]
pub struct ConnectionRowText;
#[derive(Component)]
pub struct StatRowText;
#[derive(Component)]
pub struct RecipeNameText;
#[derive(Component)]
pub struct RecipeChangeButton;
#[derive(Component)]
pub struct HpBarFill;
#[derive(Component)]
pub struct HpText;
#[derive(Component)]
pub struct AlertText;
#[derive(Component)]
pub struct PowerStatusText;

// ── Recipe selector sub-window ──

#[derive(Component)]
pub struct RecipeSelectorRoot;
#[derive(Component)]
pub struct RecipeSelectorItem {
    pub recipe_id: String,
}
#[derive(Component)]
pub struct RecipeCategoryLabel;

// ── Discovery & Archive ──

#[derive(Component, Default)]
pub struct ProductionCounter(pub u32);

#[derive(Component, Default)]
pub struct DiscoveredRecipes(pub Vec<String>);

#[derive(Component)]
pub struct Archive;

// ── Sorter settings (kept from old UI) ──

#[derive(Component)]
pub struct SorterResourceButton {
    pub resource: ResourceId,
}
#[derive(Component)]
pub struct SorterInvertButton;

// ── Farm panel ──

#[derive(Component)]
pub struct FarmCropText;
#[derive(Component)]
pub struct FarmCultivatorCountText;
#[derive(Component)]
pub struct FarmRecruitButton;
#[derive(Component)]
pub struct FarmCropSelectButton {
    pub crop_type: String,
}
