// Ouverture/fermeture + interactions du panneau d'inspection.
// Les mises à jour live se font via les systèmes dans src/ui/components/ (update_capsule_statuses, etc.)
// La génération du contenu se fait via TOML + layout_engine dans ui/panels/.

pub mod interaction;

pub use interaction::*;

use crate::core::input::KeyBindings;
use crate::core::utils::silent_despawn;
use crate::economy::components::{
    Active, ActiveToggleButton, BuildingPanel, CloseButton, PanelOverlay,
};
use bevy::prelude::*;

// ── Open / close panel ──

fn close_panel_impl(commands: &mut Commands, panel: &mut BuildingPanel) {
    if let Some(e) = panel.root.take() {
        silent_despawn(commands, e);
    }
    if let Some(e) = panel.overlay.take() {
        silent_despawn(commands, e);
    }
    panel.inspected = None;
    panel.dirty = false;
}

#[allow(unused_mut)]
pub fn close_panel(mut commands: Commands, mut panel: ResMut<BuildingPanel>) {
    close_panel_impl(&mut commands, &mut panel);
}

// ── Overlay click to close ──

pub fn overlay_click_system(
    commands: Commands,
    panel: ResMut<BuildingPanel>,
    q: Query<&Interaction, (Changed<Interaction>, With<PanelOverlay>)>,
) {
    for interaction in &q {
        if *interaction != Interaction::Pressed { continue; }
        close_panel(commands, panel);
        return;
    }
}

// ── Close button ──

#[allow(unused_mut)]
pub fn close_button_system(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    query: Query<&Interaction, (Changed<Interaction>, With<CloseButton>)>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed { continue; }
        close_panel(commands, panel);
        return;
    }
}

// ── Escape to close ──

#[allow(unused_mut)]
pub fn close_popup_on_escape(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
) {
    if !bindings.just_pressed("cancel", &keys, &mouse) { return; }
    if panel.overlay.is_some() { close_panel(commands, panel); }
}

// ── Active toggle ──

pub fn active_toggle_system(
    panel: ResMut<BuildingPanel>,
    query: Query<&Interaction, (Changed<Interaction>, With<ActiveToggleButton>)>,
    mut active_query: Query<&mut Active>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed { continue; }
        let Some(inspected) = panel.inspected else { continue; };
        if let Ok(mut active) = active_query.get_mut(inspected) {
            active.0 = !active.0;
        }
    }
}

// ── Cleanup on state exit ──

pub fn cleanup_popup(mut commands: Commands, query: Query<Entity, With<PanelOverlay>>) {
    for entity in &query {
        silent_despawn(&mut commands, entity);
    }
}

// ── Cache capsule/objective data for UiDataContext ──

pub fn cache_capsule_ui_data(
    obj: Res<crate::player::objective::ObjectiveState>,
    mut panel: ResMut<BuildingPanel>,
) {
    panel.cached_objective = obj.active_text.clone();
}
