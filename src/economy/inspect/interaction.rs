#![allow(clippy::type_complexity)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::drop_non_drop)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::useless_format)]
#![allow(clippy::single_match)]
// Click handling on building inspection panels. Spawns TOML-driven panels via LayoutEngine.
// et reproduis ici seulement si nécessaire pour la rétrocompatibilité.

use crate::agriculture::components::Cultivator;
use crate::core::modding::ModRegistry;
use crate::core::toast::ToastQueue;
use crate::core::utils::silent_despawn;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{
    Assembler, BuildMode, Building, BuildingPanel, FarmCropSelectButton, FarmRecruitButton,
    OccupiedTiles, Player, ProductionCounter, Sorter, SorterInvertButton,
    SorterResourceButton,
};
use crate::economy::game_components::{Capsule, CurrentTier, Level};
use crate::economy::recipe::RecipeRegistry;
use crate::economy::resource::{Inventory, ResourceRegistry};
use crate::economy::spatial::SpatialRegistry;
use crate::economy::ui_components::UpgradeButton;
use crate::economy::unit_config::UnitConfig;
use crate::map::components::{TilePosition, cursor_to_tile};
use crate::map::config::MapConfig;
use crate::rendering::minimap::MinimapCamera;
use crate::ui::context::UiDataContext;
use crate::ui::engine::LayoutEngine;
use bevy::prelude::*;

use crate::ui::types::PanelType;
use crate::ui::panels::PanelRegistry;
use super::close_panel;

pub fn not_build_mode(build_mode: Res<BuildMode>) -> bool {
    build_mode.0.is_none()
}

// ── Click detection ──

#[allow(clippy::too_many_arguments)]
pub fn building_inspect_click(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<MinimapCamera>)>,
    cfg: Res<MapConfig>,
    spatial: Res<SpatialRegistry>,
    building_query: Query<(&Building, Option<&OccupiedTiles>)>,
    tier_q: Query<&CurrentTier, With<Capsule>>,
    panel_registry: Res<PanelRegistry>,
    mods: Res<ModRegistry>,
    layout_engine: Res<LayoutEngine>,
    reg: Res<BuildingRegistry>,
    resource_registry: Res<ResourceRegistry>,
    asm_q: Query<&Assembler>,
    recipes: Res<RecipeRegistry>,
) {
    if !buttons.just_pressed(MouseButton::Left) { return; }

    let Some(TilePosition { x: tile_x, y: tile_y }) =
        cursor_to_tile(&windows, &camera, &cfg)
    else {
        return;
    };

    if let Some(entity) = spatial.at(tile_x, tile_y) {
        if panel.inspected == Some(entity) {
            close_panel(commands, panel);
            return;
        }

        let Ok((building, _occupied)) = building_query.get(entity) else {
            return;
        };

        let panel_type = if tier_q.contains(entity) {
            PanelType::Capsule
        } else {
            PanelType::Building
        };

        if let Some(panel_impl) = panel_registry.get(&panel_type) {
            let mut panel_data = std::collections::HashMap::new();
            panel_data.insert("building.name".into(), building.name.clone());
            panel_data.insert("building.kind".into(), building.kind.clone());

            if let Ok(tier) = tier_q.get(entity) {
                panel_data.insert("tier.current".into(), tier.0.to_string());
                panel_data.insert("capsule.current_tier".into(), tier.0.to_string());
                if let Some(def) = reg.get(&building.kind) {
                    let total_tiers = if def.tiers.len() > 1 { def.tiers.len() - 1 } else { 0 };
                    panel_data.insert("capsule.total_tiers".into(), total_tiers.to_string());
                }
            }

            panel_data.insert("objective.current".into(), panel.cached_objective.clone());

            // Pre-resolve capsule.phase_list as TOML string
            if let Ok(tier) = tier_q.get(entity) {
                let phase_names = [
                    "Phase 0 (Réveil)",
                    "Phase 1 (Étincelle)",
                    "Phase 2 (Rouille & Vapeur)",
                    "Phase 3 (Fil du Cuivre)",
                    "Phase 4 (Pouls)",
                    "Phase 5 (Nanites)",
                    "Phase 6 (Genèse)",
                    "SÉQUENCE FINALE",
                    "APOTHÉOSE",
                ];
                let items: Vec<String> = phase_names.iter().enumerate().map(|(i, name)| {
                    let state = if i < tier.0 { "done" } else if i == tier.0 { "current" } else { "locked" };
                    let sep = if i == 7 || i == 8 { "separator = true," } else { "" };
                    format!("{{id = \"{}\", title = \"{}\", state = \"{}\", {}}}", i, name, state, sep)
                }).collect();
                panel_data.insert("capsule.phase_list".into(), format!("items = [{}]", items.join(",")));
            }

            if let Ok(asm) = asm_q.get(entity) {
                if let Some(def) = recipes.get(&asm.recipe_id) {
                    panel_data.insert("recipe.name".into(), resource_registry.display_name(&crate::economy::resource::ResourceId::new(&asm.recipe_id)).to_string());
                    let inputs: Vec<String> = def.input.iter()
                        .map(|(rid, amt)| format!("{} {}×", resource_registry.display_name(rid), amt))
                        .collect();
                    panel_data.insert("recipe.inputs".into(), inputs.join(" + "));
                    let outputs: Vec<String> = def.output.iter()
                        .map(|(rid, amt)| format!("{} {}×", resource_registry.display_name(rid), amt))
                        .collect();
                    panel_data.insert("recipe.outputs".into(), outputs.join(" + "));
                    let progress = if def.time_sec > 0.0 {
                        (asm.production_timer / def.time_sec).min(1.0)
                    } else { 0.0 };
                    panel_data.insert("recipe.progress".into(), progress.to_string());
                    panel_data.insert("recipe.time_sec".into(), def.time_sec.to_string());
                }
            }

            let data_ctx = UiDataContext::new(entity, panel_data);
            let pctx = crate::ui::panels::PanelSpawnCtx {
                entity,
                building_kind: &building.kind,
                building_registry: &reg,
                resource_registry: &resource_registry,
                data: &data_ctx,
                mods: &mods,
                layout_engine: &layout_engine,
            };
            panel_impl.spawn(&mut commands, &mut panel, &pctx);
        }
    }
}

// ── Sorter resource button click ──

pub fn sorter_resource_click_system(
    mut panel: ResMut<BuildingPanel>,
    query: Query<(&Interaction, &SorterResourceButton), Changed<Interaction>>,
    mut sorter_query: Query<&mut Sorter>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    for (interaction, btn) in &query {
        if *interaction != Interaction::Pressed { continue; }
        let Some(inspected) = panel.inspected else { continue; };
        if let Ok(mut sorter) = sorter_query.get_mut(inspected) {
            sorter.filter = btn.resource.clone();
            toast_queue.0.push(format!("Sorter filter: {}", btn.resource.display_name()));
            panel.dirty = true;
        }
    }
}

// ── Sorter invert button click ──

pub fn sorter_invert_click_system(
    mut panel: ResMut<BuildingPanel>,
    query: Query<&Interaction, (Changed<Interaction>, With<SorterInvertButton>)>,
    mut sorter_query: Query<&mut Sorter>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed { continue; }
        let Some(inspected) = panel.inspected else { continue; };
        if let Ok(mut sorter) = sorter_query.get_mut(inspected) {
            sorter.inverted = !sorter.inverted;
            let mode = if sorter.inverted { "inverted" } else { "normal" };
            toast_queue.0.push(format!("Sorter: {}", mode));
            panel.dirty = true;
        }
    }
}

// ── Farm crop select button ──

pub fn farm_crop_select_system(
    query: Query<(&Interaction, &FarmCropSelectButton), Changed<Interaction>>,
    mut farm_query: Query<&mut crate::agriculture::components::Farm>,
    panel: Res<BuildingPanel>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    for (interaction, btn) in &query {
        if *interaction != Interaction::Pressed { continue; }
        let Some(inspected) = panel.inspected else { continue; };
        if let Ok(mut farm) = farm_query.get_mut(inspected) {
            let idx = farm.crop_types.iter().position(|c| c == &btn.crop_type);
            if let Some(i) = idx {
                farm.crop_index = i;
                toast_queue.0.push(format!("Crop: {}", btn.crop_type));
            }
        }
    }
}

// ── Farm recruit button ──

#[allow(clippy::too_many_arguments)]
pub fn farm_recruit_system(
    mut commands: Commands,
    panel: ResMut<BuildingPanel>,
    query: Query<&Interaction, (Changed<Interaction>, With<FarmRecruitButton>)>,
    farm_query: Query<&crate::agriculture::components::Farm>,
    farm_tf_query: Query<&Transform, With<crate::agriculture::components::Farm>>,
    unit_cfg: Res<UnitConfig>,
    mut player_inv_query: Query<&mut Inventory, With<Player>>,
    cfg: Res<MapConfig>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed { continue; }
        let Some(inspected) = panel.inspected else { continue; };
        if farm_query.get(inspected).is_err() { continue; }

        let Some(def) = unit_cfg.get("cultivator") else { continue; };
        let mut player_inv = match player_inv_query.single_mut() {
            Ok(inv) => inv,
            Err(_) => continue,
        };
        let can_afford = def.cost.iter().all(|c| player_inv.get(&c.resource) >= c.amount);
        if !can_afford {
            toast_queue.0.push("Not enough resources".to_string());
            continue;
        }
        for c in &def.cost {
            player_inv.remove(&c.resource, c.amount);
        }

        let tile_size = cfg.tile_size;
        let spawn_pos = if let Ok(tf) = farm_tf_query.get(inspected) {
            tf.translation + Vec3::new(tile_size * 0.8, 0.0, 0.5)
        } else {
            Vec3::new(0.0, 0.0, 2.5)
        };
        commands.spawn((
            Cultivator {
                state: crate::agriculture::components::CultivatorState::Idle,
                carried_resource: None,
                carried_amount: 0,
                carry_capacity: def.carry_capacity,
            },
            crate::economy::components::Unit,
            crate::enemy::components::Health { current: def.hp, max: def.hp },
            Transform::from_translation(spawn_pos),
        ));
        toast_queue.0.push("Cultivator recruited".to_string());
    }
}

// ── Upgrade button click ──

#[allow(clippy::too_many_arguments)]
pub fn upgrade_button_system(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    query: Query<&Interaction, (Changed<Interaction>, With<UpgradeButton>)>,
    upgrade_query: Query<&UpgradeButton>,
    building_query: Query<(&Building, &TilePosition, &OccupiedTiles)>,
    mut player_inv_query: Query<&mut Inventory, With<Player>>,
    registry: Res<BuildingRegistry>,
    cfg: Res<MapConfig>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    let Some(inspected) = panel.inspected else { return; };

    let mut pressed = false;
    for interaction in &query {
        if *interaction == Interaction::Pressed && upgrade_query.get(inspected).is_ok() {
            pressed = true; break;
        }
    }
    if !pressed { return; }

    let Ok((building, tile_pos, occupied)) = building_query.get(inspected) else { return; };
    let Ok(upgrade_btn) = upgrade_query.get(inspected) else { return; };
    let target_kind = &upgrade_btn.target_kind;
    let Some(target_def) = registry.get(target_kind) else {
        toast_queue.0.push("Upgrade target not found".to_string());
        return;
    };

    let mut player_inv = match player_inv_query.single_mut() {
        Ok(inv) => inv,
        Err(_) => return,
    };
    let can_afford = target_def.cost.iter().all(|c| player_inv.get(&c.resource) >= c.amount);
    if !can_afford {
        toast_queue.0.push("Not enough resources to upgrade".to_string());
        return;
    }
    for c in &target_def.cost { player_inv.remove(&c.resource, c.amount); }

    let old_name = building.name.clone();
    let tx = tile_pos.x;
    let ty = tile_pos.y;
    let footprint = occupied.0.clone();
    let tile_size = cfg.tile_size;
    let (tw, th) = target_def.tile_size;
    let cx = (tx as f32 + (tw as f32 - 1.0) * 0.5) * tile_size;
    let cy = (ty as f32 + (th as f32 - 1.0) * 0.5) * tile_size;

    drop(player_inv);
    if let Some(e) = panel.root.take() { silent_despawn(&mut commands, e); }
    if let Some(e) = panel.overlay.take() { silent_despawn(&mut commands, e); }
    panel.inspected = None;
    panel.dirty = false;

    silent_despawn(&mut commands, inspected);

    let mut e = commands.spawn((
        Building { kind: target_kind.clone(), name: target_def.name.clone() },
        OccupiedTiles(footprint),
        TilePosition { x: tx, y: ty },
        Transform::from_xyz(cx, cy, 2.0),
        Level(target_def.level),
    ));

    if let Some(ref recipe) = target_def.default_recipe {
        let interval = target_def.production_interval.unwrap_or(2.0);
        e.insert(Assembler { production_timer: 0.0, interval, recipe_id: recipe.clone() });
        e.insert(ProductionCounter::default());
        e.insert(crate::economy::components::DiscoveredRecipes::default());
    }
    if let Some(ref stats) = target_def.combat {
        e.insert(crate::economy::components::TurretCombat {
            damage: stats.damage, range_sq: stats.range,
            fire_interval: stats.fire_rate_sec, timer: 0.0,
            projectile_speed: stats.projectile_speed,
        });
    }
    if target_def.inventory_capacity > 0 {
        e.insert(Inventory::with_capacity(target_def.inventory_capacity));
    } else {
        e.insert(Inventory::new());
    }
    crate::economy::building::attach_power_components(&mut e, target_def);

    toast_queue.0.push(format!("Upgraded {} to {}", old_name, target_def.name));
}