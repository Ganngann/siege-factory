use crate::economy::resource::ResourceId;
use bevy::prelude::*;

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
    pub source_owner: Option<Entity>,
    pub source_slot_index: usize,
    pub resource: Option<crate::economy::resource::ResourceId>,
    pub amount: u32,
    pub visual: Option<Entity>,
}

impl DragState {
    pub fn reset(&mut self) {
        self.active = false;
        self.source_owner = None;
        self.source_slot_index = 0;
        self.resource = None;
        self.amount = 0;
        self.visual = None;
    }
}

#[derive(Resource, Default)]
pub struct UiIsBlocking(pub bool);

#[derive(Resource, Default)]
pub struct BuildingPanel {
    pub inspected: Option<Entity>,
    pub root: Option<Entity>,
    pub overlay: Option<Entity>,
    pub recipe_selector: Option<Entity>,
    pub dirty: bool,
}

// ── Build state resources ──

#[derive(Resource, Default)]
pub struct BuildMode(pub Option<String>);

#[derive(Resource, Default)]
pub struct BeltDirection(pub crate::economy::game_components::Direction);

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
pub struct FuelBarBg;
#[derive(Component)]
pub struct FuelBarFill;
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

// ── Sorter settings ──

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

// ── Upgrade panel ──

#[derive(Component)]
pub struct UpgradeButton {
    pub target_kind: String,
}
#[derive(Component)]
pub struct UpgradeInfoText;
