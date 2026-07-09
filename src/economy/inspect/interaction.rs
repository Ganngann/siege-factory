use crate::agriculture::components::Cultivator;
use crate::core::toast::ToastQueue;
use crate::core::utils::silent_despawn;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{
    BuildMode, Building, BuildingPanel, DeconstructMode, FarmCropSelectButton, FarmRecruitButton,
    OccupiedTiles, Player, Sorter, SorterInvertButton,
    SorterResourceButton, UiIsBlocking,
};
use crate::economy::game_components::{Capsule, CurrentTier, Level};
use crate::economy::tiered_structure::ProgressionLogRegistry;
use crate::economy::resource::{Inventory, ResourceRegistry};
use crate::economy::spatial::SpatialRegistry;
use crate::economy::ui_components::UpgradeButton;
use crate::economy::unit_config::UnitConfig;
use crate::map::components::{TilePosition, cursor_to_tile};
use crate::map::config::MapConfig;
use crate::rendering::minimap::MinimapCamera;
use bevy::prelude::*;

use super::{close_panel, open_capsule_panel, open_panel};

// ── Click detection ──

// SUGGEST: extraire dans un struct SystemParam (clippy::too_many_arguments)
pub fn building_inspect_click(
    commands: Commands,
    panel: ResMut<BuildingPanel>,
    build_mode: Res<BuildMode>,
    deconstruct: Res<DeconstructMode>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    // SUGGEST: type CameraQuery = Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<MinimapCamera>)> (clippy::type_complexity)
    camera: Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<MinimapCamera>)>,
    cfg: Res<MapConfig>,
    spatial: Res<SpatialRegistry>,
    building_query: Query<(&Building, Option<&OccupiedTiles>)>,
    tier_q: Query<&CurrentTier, With<Capsule>>,
    resource_registry: Res<ResourceRegistry>,
    reg: Res<BuildingRegistry>,
    progression_logs: Res<ProgressionLogRegistry>,
    ui_blocking: Res<UiIsBlocking>,
) {
    if ui_blocking.0 {
        return;
    }
    if build_mode.0.is_some() || deconstruct.0 {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(TilePosition {
        x: tile_x,
        y: tile_y,
    }) = cursor_to_tile(&windows, &camera, &cfg)
    else {
        return;
    };

    // Check buildings first (they occupy tiles in SpatialRegistry)
    if let Some(entity) = spatial.at(tile_x, tile_y) {
        if panel.inspected == Some(entity) {
            close_panel(commands, panel);
            return;
        }

        let Ok((building, _occupied)) = building_query.get(entity) else {
            return;
        };

        // Check if this building is a Capsule with tiers → show capsule panel
        let building_def = reg.get(&building.kind);
        if tier_q.contains(entity) && building_def.map_or(false, |d| !d.tiers.is_empty()) {
            let current_tier = tier_q
                .get(entity)
                .map(|t| t.0)
                .unwrap_or(0);
            open_capsule_panel(
                commands,
                panel,
                entity,
                building,
                &reg,
                &progression_logs,
                current_tier,
            );
            return;
        }

        let farm_crop_types = building_def
            .map(|d| d.crop_types.clone())
            .unwrap_or_default();
        open_panel(
            commands,
            panel,
            entity,
            building,
            &building.kind,
            &resource_registry,
            &reg,
            farm_crop_types,
        );
        return;
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
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(inspected) = panel.inspected else {
            continue;
        };
        if let Ok(mut sorter) = sorter_query.get_mut(inspected) {
            sorter.filter = btn.resource.clone();
            toast_queue
                .0
                .push(format!("Sorter filter: {}", btn.resource.display_name()));
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
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(inspected) = panel.inspected else {
            continue;
        };
        if let Ok(mut sorter) = sorter_query.get_mut(inspected) {
            sorter.inverted = !sorter.inverted;
            let mode = if sorter.inverted {
                "inverted"
            } else {
                "normal"
            };
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
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(inspected) = panel.inspected else {
            continue;
        };
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

// SUGGEST: extraire dans un struct SystemParam (clippy::too_many_arguments)
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
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(inspected) = panel.inspected else {
            continue;
        };
        if farm_query.get(inspected).is_err() {
            continue;
        }

        let Some(def) = unit_cfg.get("cultivator") else {
            continue;
        };
        let mut player_inv = match player_inv_query.single_mut() {
            Ok(inv) => inv,
            Err(_) => continue,
        };
        let can_afford = def
            .cost
            .iter()
            .all(|c| player_inv.get(&c.resource) >= c.amount);
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
            crate::enemy::components::Health {
                current: def.hp,
                max: def.hp,
            },
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
    let Some(inspected) = panel.inspected else {
        return;
    };

    // Check if upgrade button was pressed
    let mut pressed = false;
    for interaction in &query {
        if *interaction == Interaction::Pressed
            && upgrade_query.get(inspected).is_ok() {
                pressed = true;
                break;
            }
    }
    if !pressed {
        return;
    }

    let Ok((building, tile_pos, occupied)) = building_query.get(inspected) else {
        return;
    };
    let Ok(upgrade_btn) = upgrade_query.get(inspected) else {
        return;
    };

    let target_kind = &upgrade_btn.target_kind;
    let Some(target_def) = registry.get(target_kind) else {
        toast_queue.0.push("Upgrade target not found".to_string());
        return;
    };

    // Check cost
    let mut player_inv = match player_inv_query.single_mut() {
        Ok(inv) => inv,
        Err(_) => return,
    };
    let can_afford = target_def
        .cost
        .iter()
        .all(|c| player_inv.get(&c.resource) >= c.amount);
    if !can_afford {
        toast_queue
            .0
            .push("Not enough resources to upgrade".to_string());
        return;
    }

    // Deduct cost
    for c in &target_def.cost {
        player_inv.remove(&c.resource, c.amount);
    }

    let old_name = building.name.clone();
    let tx = tile_pos.x;
    let ty = tile_pos.y;
    let footprint = occupied.0.clone();
    let tile_size = cfg.tile_size;
    let (tw, th) = target_def.tile_size;
    let cx = (tx as f32 + (tw as f32 - 1.0) * 0.5) * tile_size;
    let cy = (ty as f32 + (th as f32 - 1.0) * 0.5) * tile_size;

    // Close panel
    // SUGGEST: drop() ici n'étend pas la durée de vie du borrow — le drop est inutile (clippy::drop_non_drop)
    drop(player_inv);
    // Inline panel close (can't call close_panel as it moves commands/panel)
    if let Some(e) = panel.root.take() {
        silent_despawn(&mut commands, e);
    }
    if let Some(e) = panel.overlay.take() {
        silent_despawn(&mut commands, e);
    }
    if let Some(e) = panel.recipe_selector.take() {
        silent_despawn(&mut commands, e);
    }
    panel.inspected = None;
    panel.dirty = false;

    // Despawn old entity
    silent_despawn(&mut commands, inspected);

    // Spawn upgraded building at the same position
    let mut e = commands.spawn((
        Building {
            kind: target_kind.clone(),
            name: target_def.name.clone(),
        },
        OccupiedTiles(footprint),
        TilePosition { x: tx, y: ty },
        Transform::from_xyz(cx, cy, 2.0),
        Level(target_def.level),
    ));

    // Attach production if the target has a default recipe
    if let Some(ref recipe) = target_def.default_recipe {
        let interval = target_def.production_interval.unwrap_or(2.0);
        e.insert(crate::economy::components::Assembler {
            production_timer: 0.0,
            interval,
            recipe_id: recipe.clone(),
        });
        e.insert(crate::economy::components::ProductionCounter::default());
        e.insert(crate::economy::components::DiscoveredRecipes::default());
    }

    // Attach combat if target has combat stats
    if let Some(ref stats) = target_def.combat {
        e.insert(crate::economy::components::TurretCombat {
            damage: stats.damage,
            range_sq: stats.range,
            fire_interval: stats.fire_rate_sec,
            timer: 0.0,
            projectile_speed: stats.projectile_speed,
        });
    }

    // Attach inventory
    if target_def.inventory_capacity > 0 {
        e.insert(Inventory::with_capacity(target_def.inventory_capacity));
    } else {
        e.insert(Inventory::new());
    }

    // Attach power components
    crate::economy::building::attach_power_components(&mut e, target_def);

    toast_queue
        .0
        .push(format!("Upgraded {} to {}", old_name, target_def.name));
}
