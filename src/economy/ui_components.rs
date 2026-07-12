// 🏗️ LEGACY UI — ces composants sont progressivement remplacés par src/ui/components/ et src/ui/types.rs.
// - InventoryGrid, InventorySlot, DragState, DraggedItemVisual → ui/components/inventory_grid.rs
// - BuildingPanel, PanelOverlay, PanelModal → ui/types.rs
// - Tous les *Button, *Text, *Fill → ui/components/ (label, progress_bar, hp_bar, etc.)
// Si tu ajoutes un composant UI, ajoute-le dans ui/components/ avec le trait UiComponent.
// 🪣 IA NOTE: les composants legacy sont re-exportés ici.
// Si tu ajoutes un composant dans ui/components/, n'oublie pas le `pub use` ici.

use bevy::prelude::*;
use crate::economy::resource::ResourceId;

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

#[derive(Component)]
pub struct ManagedByPanel;

#[derive(Resource, Default)]
pub struct UiIsBlocking(pub bool);

#[derive(Resource, Default)]
pub struct BuildingPanel {
    pub inspected: Option<Entity>,
    pub root: Option<Entity>,
    pub overlay: Option<Entity>,
    pub dirty: bool,
    pub cached_objective: String,
    pub cached_phase_list: String,
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
pub struct DragHandle;
#[derive(Component)]
pub struct ActiveToggleButton;
#[derive(Component)]
pub struct CloseButton;

// ── Sorter settings ──

#[derive(Component)]
pub struct SorterResourceButton {
    pub resource: ResourceId,
}
#[derive(Component)]
pub struct SorterInvertButton;

// ── Farm panel ──

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
