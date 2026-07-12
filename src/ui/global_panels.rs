use bevy::prelude::*;
use crate::core::modding::ModRegistry;
use crate::core::utils::silent_despawn;
use crate::economy::components::Player;
use crate::enemy::components::WaveState;
use crate::ui::context::UiDataContext;
use crate::ui::engine::LayoutEngine;

// ── Components ──

#[derive(Component)]
pub struct GameOverOverlay;

#[derive(Component)]
pub struct InventoryPanelRoot;

#[derive(Component)]
pub struct CraftingPanelRoot;

// ── Game Over ──

pub fn spawn_game_over_overlay(
    mut commands: Commands,
    mods: Res<ModRegistry>,
    engine: Res<LayoutEngine>,
    wave: Res<WaveState>,
) {
    let Some(content) = mods.load_data("panel_game_over.toml") else { return };
    let Ok(config) = toml::from_str::<toml::Value>(&content) else { return };

    commands.spawn((Camera2d, GameOverOverlay));

    let mut data = std::collections::HashMap::new();
    let waves_survived = wave.wave.saturating_sub(1);
    data.insert("game_over.waves".into(), format!("Waves survived: {}", waves_survived));

    let dummy = commands.spawn_empty().id();
    let ctx = UiDataContext::new(dummy, data);
    let root = engine.render_fullscreen(&mut commands, &config, &ctx);
    commands.entity(root).insert(GameOverOverlay);
}

pub fn despawn_game_over_overlay(
    mut commands: Commands,
    root_q: Query<Entity, With<GameOverOverlay>>,
) {
    for e in &root_q { silent_despawn(&mut commands, e); }
}

// ── Inventory (I key) ──

pub fn toggle_inventory(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    panel_q: Query<Entity, With<InventoryPanelRoot>>,
    player_q: Query<Entity, With<Player>>,
    mods: Res<ModRegistry>,
    engine: Res<LayoutEngine>,
) {
    if !keys.just_pressed(KeyCode::KeyI) { return; }

    if let Ok(e) = panel_q.single() {
        silent_despawn(&mut commands, e);
        return;
    }

    let Ok(player) = player_q.single() else { return };

    let Some(content) = mods.load_data("panel_inventory.toml") else { return };
    let Ok(config) = toml::from_str::<toml::Value>(&content) else { return };

    let data = UiDataContext::new(player, Default::default());
    let (overlay, root) = engine.render_panel(&mut commands, &config, player, &data);
    commands.entity(root).insert(InventoryPanelRoot);
    commands.entity(overlay).insert(InventoryPanelRoot);
}

pub fn cleanup_inventory(
    mut commands: Commands,
    panel_q: Query<Entity, With<InventoryPanelRoot>>,
) {
    for e in &panel_q { silent_despawn(&mut commands, e); }
}

// ── Hand Crafting (C key) ──

pub fn toggle_crafting(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    panel_q: Query<Entity, With<CraftingPanelRoot>>,
    player_q: Query<Entity, With<Player>>,
    mods: Res<ModRegistry>,
    engine: Res<LayoutEngine>,
) {
    if !keys.just_pressed(KeyCode::KeyC) { return; }

    if let Ok(e) = panel_q.single() {
        silent_despawn(&mut commands, e);
        return;
    }

    let Ok(player) = player_q.single() else { return };

    let Some(content) = mods.load_data("panel_hand_crafting.toml") else { return };
    let Ok(config) = toml::from_str::<toml::Value>(&content) else { return };

    let data = UiDataContext::new(player, Default::default());
    let (overlay, root) = engine.render_panel(&mut commands, &config, player, &data);
    commands.entity(root).insert(CraftingPanelRoot);
    commands.entity(overlay).insert(CraftingPanelRoot);
}

pub fn cleanup_crafting(
    mut commands: Commands,
    panel_q: Query<Entity, With<CraftingPanelRoot>>,
) {
    for e in &panel_q { silent_despawn(&mut commands, e); }
}
