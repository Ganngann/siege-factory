pub mod deposit_panel;
pub mod interaction;
pub mod recipe_selector;
pub mod spawn;
pub mod update;

pub use deposit_panel::*;
pub use interaction::*;
pub use recipe_selector::*;
pub use spawn::*;
pub use update::*;

use crate::core::input::KeyBindings;
use crate::core::utils::silent_despawn;
use crate::economy::components::{
    Active, ActiveToggleButton, BuildingPanel, CloseButton, PanelModal, PanelOverlay,
};
use bevy::prelude::*;

const MODAL_WIDTH: f32 = 800.0;
const MODAL_HEIGHT: f32 = 560.0;
const DEPOSIT_MODAL_WIDTH: f32 = 400.0;
const DEPOSIT_MODAL_HEIGHT: f32 = 200.0;
const RECIPE_SELECTOR_WIDTH: f32 = 420.0;
const RECIPE_SELECTOR_HEIGHT: f32 = 300.0;

const SECTION_FONT_SIZE: f32 = 11.0;
const BAR_HEIGHT: f32 = 12.0;
const CLOSE_BUTTON_SIZE: f32 = 26.0;
const CLOSE_BUTTON_FONT: f32 = 14.0;

// ── Open / close panel ──

fn close_panel_impl(commands: &mut Commands, panel: &mut BuildingPanel) {
    if let Some(e) = panel.root.take() {
        silent_despawn(commands, e);
    }
    if let Some(e) = panel.overlay.take() {
        silent_despawn(commands, e);
    }
    if let Some(e) = panel.recipe_selector.take() {
        silent_despawn(commands, e);
    }
    panel.inspected = None;
    panel.dirty = false;
}

pub fn close_panel(mut commands: Commands, mut panel: ResMut<BuildingPanel>) {
    close_panel_impl(&mut commands, &mut panel);
}

// ── Overlay click to close ──

pub fn overlay_click_system(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    // SUGGEST: type ModalQuery = Query<(&Node, &GlobalTransform), (With<PanelModal>, Without<PanelOverlay>)> (clippy::type_complexity)
    modal_query: Query<(&Node, &GlobalTransform), (With<PanelModal>, Without<PanelOverlay>)>,
) {
    if panel.overlay.is_none() {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    // If click inside the modal body → let the modal's own buttons handle it
    if let Ok((_node, transform)) = modal_query.single() {
        let center = transform.translation().truncate();
        let modal_rect = Rect::from_center_size(center, Vec2::new(MODAL_WIDTH, MODAL_HEIGHT));
        if modal_rect.contains(cursor) {
            return;
        }
    } else {
        return;
    }

    // Click is outside the modal → close
    if panel.recipe_selector.is_some() {
        if let Some(e) = panel.recipe_selector.take() {
            silent_despawn(&mut commands, e);
        }
    } else {
        close_panel(commands, panel);
    }
}

// ── Close button ──

pub fn close_button_system(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    query: Query<&Interaction, (Changed<Interaction>, With<CloseButton>)>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if panel.recipe_selector.is_some() {
            if let Some(e) = panel.recipe_selector.take() {
                silent_despawn(&mut commands, e);
            }
        } else {
            close_panel(commands, panel);
        }
        return;
    }
}

// ── Escape to close ──

pub fn close_popup_on_escape(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
) {
    if !bindings.just_pressed("cancel", &keys, &mouse) {
        return;
    }
    if panel.recipe_selector.is_some() {
        if let Some(e) = panel.recipe_selector.take() {
                silent_despawn(&mut commands, e);
            }
        } else if panel.overlay.is_some() {
        close_panel(commands, panel);
    }
}

// ── Active toggle ──

pub fn active_toggle_system(
    panel: ResMut<BuildingPanel>,
    query: Query<&Interaction, (Changed<Interaction>, With<ActiveToggleButton>)>,
    mut active_query: Query<&mut Active>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(inspected) = panel.inspected else {
            continue;
        };
        if let Ok(mut active) = active_query.get_mut(inspected) {
            active.0 = !active.0;
        }
    }
}

// ── Cleanup on state exit ──

pub fn cleanup_popup(mut commands: Commands, query: Query<Entity, With<PanelModal>>) {
    for entity in &query {
        silent_despawn(&mut commands, entity);
    }
}
