// 🏗️ LEGACY UI — barre de construction en bas de l'écran.
// Pas encore migrée vers src/ui/. À migrer quand le nouveau système
// supportera les layouts ancrés (bottom: 0).

pub mod interaction;
pub mod menu;

pub use interaction::*;
pub use menu::*;

use crate::core::utils::silent_despawn;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::MenuBarPanel;
use crate::economy::discovery::GlobalArchive;
use crate::economy::menu::{MenuDef, MenuItems, MenuState};
use crate::economy::unit_config::UnitConfig;
use crate::rendering::TextureCache;
use bevy::prelude::*;

const PANEL_HEIGHT: f32 = 90.0;
const ITEM_WIDTH: f32 = 90.0;
const ITEM_HEIGHT: f32 = 70.0;
const SCROLL_BUTTON_WIDTH: f32 = 24.0;
const BACK_BUTTON_WIDTH: f32 = 60.0;
const BORDER_WIDTH: f32 = 2.0;

// SUGGEST: extraire dans un struct SystemParam (clippy::too_many_arguments)
pub fn spawn_menu_bar(
    mut commands: Commands,
    menu_def: Res<MenuDef>,
    menu_state: Res<MenuState>,
    mut menu_items: ResMut<MenuItems>,
    registry: Res<BuildingRegistry>,
    unit_cfg: Res<UnitConfig>,
    textures: Res<TextureCache>,
    global_archive: Res<GlobalArchive>,
) {
    *menu_items = crate::economy::menu::flat_items_at(
        &menu_def.root,
        &menu_state.stack,
        menu_state.scroll,
        &registry,
        &unit_cfg,
        &menu_def,
        &global_archive,
    );

    build_menu_bar(&mut commands, &menu_items, &textures);
}

// SUGGEST: extraire dans un struct SystemParam (clippy::too_many_arguments)
pub fn refresh_menu_bar(
    mut commands: Commands,
    menu_def: Res<MenuDef>,
    menu_state: Res<MenuState>,
    mut menu_items: ResMut<MenuItems>,
    registry: Res<BuildingRegistry>,
    unit_cfg: Res<UnitConfig>,
    textures: Res<TextureCache>,
    global_archive: Res<GlobalArchive>,
    panel_query: Query<Entity, With<MenuBarPanel>>,
) {
    let new_items = crate::economy::menu::flat_items_at(
        &menu_def.root,
        &menu_state.stack,
        menu_state.scroll,
        &registry,
        &unit_cfg,
        &menu_def,
        &global_archive,
    );
    if *menu_items == new_items {
        return;
    }
    *menu_items = new_items;

    for entity in &panel_query {
        silent_despawn(&mut commands, entity);
    }
    build_menu_bar(&mut commands, &menu_items, &textures);
}

impl PartialEq for MenuItems {
    fn eq(&self, other: &Self) -> bool {
        self.items.len() == other.items.len()
            && self.items.iter().zip(other.items.iter()).all(|(a, b)| {
                a.label == b.label
                    && std::mem::discriminant(&a.kind) == std::mem::discriminant(&b.kind)
            })
            && self.has_back == other.has_back
            && self.breadcrumb == other.breadcrumb
            && self.can_scroll_left == other.can_scroll_left
            && self.can_scroll_right == other.can_scroll_right
            && self.total_items == other.total_items
    }
}

pub fn cleanup_menu_bar(mut commands: Commands, query: Query<Entity, With<MenuBarPanel>>) {
    for entity in &query {
        silent_despawn(&mut commands, entity);
    }
}
