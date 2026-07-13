#![allow(clippy::unnecessary_sort_by)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::type_complexity)]
#![allow(clippy::drop_non_drop)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::useless_format)]
#![allow(clippy::single_match)]
use bevy::prelude::*;

use crate::core::toast::ToastQueue;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{Building, Player, ResourceDeposit};
use crate::economy::discovery::GlobalArchive;
use crate::economy::game_components::{Capsule, CurrentTier};
use crate::economy::player::MiningTimer;
use crate::economy::resource::{Inventory, ResourceId};
use crate::economy::spatial::SpatialRegistry;
use crate::economy::tiered_structure::FinalCountdown;
use crate::economy::tool::ToolRegistry;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::map::tile_grid::ChunkGrid;

/// Centralisé toutes les interactions avec la touche E.
/// Press (just_pressed) : Capsule (tiers) → sinon mine (si dépôt)
/// Hold (pressed) : mine les dépôts
#[allow(clippy::too_many_arguments)]
pub fn contextual_interact(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut mining_timer: ResMut<MiningTimer>,
    mut player_q: Query<(&TilePosition, &mut Inventory), With<Player>>,
    spatial: Res<SpatialRegistry>,
    building_q: Query<&Building>,
    mut capsule_q: Query<&mut CurrentTier, With<Capsule>>,
    deposit_q: Query<(Entity, &ResourceDeposit, &TilePosition)>,
    cfg: Res<MapConfig>,
    mut chunk_grid: ResMut<ChunkGrid>,
    mut commands: Commands,
    tool_registry: Res<ToolRegistry>,
    mut archive: ResMut<GlobalArchive>,
    mut toasts: ResMut<ToastQueue>,
    mut countdown: ResMut<FinalCountdown>,
    registry: Res<BuildingRegistry>,
) {
    let Ok((player_tile, mut player_inv)) = player_q.single_mut() else {
        return;
    };

    let check_tiles = [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)];

    // Find the first interactive entity nearby
    let mut nearby_entity: Option<Entity> = None;
    for &(dx, dy) in &check_tiles {
        let tx = player_tile.x + dx;
        let ty = player_tile.y + dy;
        if let Some(entity) = spatial.at(tx, ty) {
            nearby_entity = Some(entity);
            break;
        }
    }

    let Some(entity) = nearby_entity else {
        // Nothing nearby — still try to mine if E is held
        // ⚠️ IA ATTENTION: KeyE en dur (interaction). Devrait utiliser le système KeyBindings.
        if keys.pressed(KeyCode::KeyE) {
            try_mine(player_tile, &time, &mut mining_timer, &mut player_inv, &deposit_q, &cfg, &mut chunk_grid, &mut commands, &tool_registry);
        }
        return;
    };

    // ── PRESS (just_pressed) ────────────────────────────────────
    // ⚠️ IA ATTENTION: KeyE en dur (interaction structure).
    if keys.just_pressed(KeyCode::KeyE) {
        // Priorité 1 : Capsule / structure à tiers
        if capsule_q.contains(entity) {
            if let Ok(building) = building_q.get(entity) {
                if let Some(def) = registry.get(&building.kind) {
                    if !def.tiers.is_empty() {
                        let current = capsule_q.get(entity).map(|t| t.0).unwrap_or(0);
                        if current >= def.tiers.len() {
                            toasts.0.push(format!("{}: fully upgraded", def.name));
                            return;
                        }
                        let tier_def = &def.tiers[current];
                        let can_afford = tier_def.required_items.iter()
                            .all(|(res, amt)| player_inv.get(res) >= *amt);
                        if !can_afford {
                            let missing: Vec<String> = tier_def.required_items.iter()
                                .filter(|(res, amt)| player_inv.get(res) < *amt)
                                .map(|(res, _)| res.display_name()).collect();
                            toasts.0.push(format!("{}: need {}", def.name, missing.join(", ")));
                            return;
                        }
                        for (res, amt) in &tier_def.required_items {
                            player_inv.remove(res, *amt);
                        }
                        if let Ok(mut ct) = capsule_q.get_mut(entity) {
                            ct.0 += 1;
                        }
                        for recipe_id in &tier_def.unlock_recipes {
                            archive.unlocked_recipes.insert(recipe_id.clone());
                        }
                        let new_tier = current + 1;
                        if new_tier == def.tiers.len() {
                            countdown.remaining_secs = 60.0;
                            countdown.running = true;
                            toasts.0.push(format!("{}: countdown final — 60s", def.name));
                        } else {
                            toasts.0.push(format!("{}: upgraded to tier {}", def.name, new_tier));
                        }
                        return;
                    }
                }
            }
        }
        // Priorité 2 : fallback → mine
        try_mine(player_tile, &time, &mut mining_timer, &mut player_inv, &deposit_q, &cfg, &mut chunk_grid, &mut commands, &tool_registry);
        return;
    }

    // ── HOLD (pressed) ──────────────────────────────────────────
    // ⚠️ IA ATTENTION: KeyE en dur (mining hold).
    if keys.pressed(KeyCode::KeyE) {
        try_mine(player_tile, &time, &mut mining_timer, &mut player_inv, &deposit_q, &cfg, &mut chunk_grid, &mut commands, &tool_registry);
    }
}

#[allow(clippy::too_many_arguments)]
fn try_mine(
    player_tile: &TilePosition,
    time: &Time,
    mining_timer: &mut MiningTimer,
    inv: &mut Inventory,
    deposit_q: &Query<(Entity, &ResourceDeposit, &TilePosition)>,
    cfg: &MapConfig,
    chunk_grid: &mut ChunkGrid,
    commands: &mut Commands,
    tool_registry: &ToolRegistry,
) {
    mining_timer.0 += time.delta_secs();
    let check_tiles = [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)];

    for (dep_entity, deposit, dep_tile) in deposit_q.iter() {
        if deposit.amount == 0 { continue; }
        let adjacent = check_tiles.iter().any(|&(dx, dy)| {
            dep_tile.x == player_tile.x + dx && dep_tile.y == player_tile.y + dy
        });
        if !adjacent { continue; }

        let mult = tool_registry.best_tool_for(&deposit.resource, inv)
            .map(|(_, m)| m).unwrap_or(1.0);
        let interval = cfg.player_mining_interval * mult;
        if mining_timer.0 < interval { continue; }

        inv.add(&ResourceId(deposit.resource.clone()), 1);

        if !cfg.infinite_deposits {
            use crate::map::tile_grid::CHUNK_SIZE;
            let cx = dep_tile.x.div_euclid(CHUNK_SIZE as i32);
            let cy = dep_tile.y.div_euclid(CHUNK_SIZE as i32);
            let dx = dep_tile.x.rem_euclid(CHUNK_SIZE as i32) as u32;
            let dy = dep_tile.y.rem_euclid(CHUNK_SIZE as i32) as u32;
            chunk_grid.set_deposit_amount(cx, cy, dx, dy, deposit.amount - 1);
            if let Ok(mut dep_cmd) = commands.get_entity(dep_entity) {
                dep_cmd.insert(ResourceDeposit {
                    resource: deposit.resource.clone(),
                    amount: deposit.amount - 1,
                });
            }
        }
        mining_timer.0 -= interval;
        return;
    }
}
