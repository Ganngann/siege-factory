use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::core::modding::ModRegistry;
use crate::core::utils::silent_despawn;
use crate::core::toast::ToastQueue;
use crate::core::utils::{tile_to_world, world_to_tile};
use crate::economy::components::{Building, Player};
use crate::economy::player::PlayerWorldPos;
use crate::economy::recipe::RecipeRegistry;
use crate::economy::resource::Inventory;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::config::VisualsConfig;
use crate::rendering::ShapeCache;

#[derive(Debug, Clone, Deserialize)]
pub struct TutorialStepDef {
    pub id: String,
    pub toast: String,
    pub condition: String,
    #[serde(default)]
    pub params: HashMap<String, String>,
    #[serde(default)]
    pub highlight: Option<HighlightDef>,
    #[serde(default)]
    pub persistent: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HighlightDef {
    #[serde(rename = "type")]
    pub target_type: String,
    #[serde(default)]
    pub target_id: Option<String>,
    #[serde(default)]
    pub tile_x: Option<i32>,
    #[serde(default)]
    pub tile_y: Option<i32>,
}

#[derive(Resource, Default)]
pub struct TutorialState {
    pub current_index: usize,
    pub completed: bool,
    pub steps: Vec<TutorialStepDef>,
}

#[derive(Resource, Default)]
pub struct TutorialConditions {
    pub player_moved_distance: f32,
    pub items_collected: u32,
    pub items_crafted: HashMap<String, u32>,
    pub buildings_placed: u32,
    pub building_types_placed: HashMap<String, u32>,
    pub has_interacted_with_structure: bool,
    pub structures_interacted: HashMap<String, bool>,
}

impl TutorialState {
    pub fn load(mods: &ModRegistry) -> Self {
        if let Some(content) = mods.load_data("tutorial.toml") {
            match toml::from_str::<TutorialStepsToml>(&content) {
                Ok(parsed) => {
                    return Self {
                        steps: parsed.steps,
                        ..default()
                    };
                }
                Err(e) => {
                    error!("Failed to parse tutorial.toml: {}", e);
                }
            }
        }
        Self::default()
    }
}

#[derive(Deserialize)]
struct TutorialStepsToml {
    steps: Vec<TutorialStepDef>,
}

pub fn track_player_movement(
    player_q: Query<&Transform, With<Player>>,
    mut conditions: ResMut<TutorialConditions>,
    mut last_pos: Local<Vec2>,
) {
    let Ok(tf) = player_q.single() else {
        return;
    };
    let pos = tf.translation.truncate();
    let dist = pos.distance(*last_pos);
    if dist > 0.01 {
        conditions.player_moved_distance += dist;
    }
    *last_pos = pos;
}

pub fn track_item_collected(
    player_q: Query<&Inventory, (With<Player>, Changed<Inventory>)>,
    mut conditions: ResMut<TutorialConditions>,
) {
    let Ok(inv) = player_q.single() else {
        return;
    };
    let total: u32 = inv.total();
    if total > conditions.items_collected {
        conditions.items_collected = total;
    }
}

pub fn track_item_crafted(
    player_q: Query<&Inventory, (With<Player>, Changed<Inventory>)>,
    recipes: Res<RecipeRegistry>,
    mut conditions: ResMut<TutorialConditions>,
) {
    let Ok(inv) = player_q.single() else {
        return;
    };
    for (recipe_id, recipe) in &recipes.recipes {
        let output_count: u32 = recipe.output.iter().map(|(_, a)| *a).sum();
        if output_count == 0 {
            continue;
        }
        let held = recipe
            .output
            .iter()
            .map(|(res, amt)| inv.get(res).min(*amt))
            .min()
            .unwrap_or(0);
        let produced = held / output_count;
        let entry = conditions
            .items_crafted
            .entry(recipe_id.clone())
            .or_insert(0);
        if produced > *entry {
            *entry = produced;
        }
    }
}

pub fn track_building_placed(
    building_q: Query<&Building, Added<Building>>,
    mut conditions: ResMut<TutorialConditions>,
) {
    for building in building_q.iter() {
        // ⚠️ IA ATTENTION: comparaison sur kind == "hq" en dur.
        // Si tu renommes "hq" dans buildings.toml, ce code casse silencieusement.
        if building.kind == "hq" {
            continue;
        }
        conditions.buildings_placed += 1;
        *conditions
            .building_types_placed
            .entry(building.kind.clone())
            .or_insert(0) += 1;
    }
}

pub fn tutorial_tick(
    mut state: ResMut<TutorialState>,
    conditions: Res<TutorialConditions>,
    mut toast_queue: ResMut<ToastQueue>,
    player_pos: Res<PlayerWorldPos>,
    building_q: Query<(&Building, &TilePosition)>,
    keys: Res<ButtonInput<KeyCode>>,
    tile_size: Res<crate::map::config::MapConfig>,
) {
    if state.completed || state.steps.is_empty() {
        return;
    }

    let step = &state.steps[state.current_index];
    let met = evaluate_condition(
        &step.condition,
        &step.params,
        &conditions,
        &player_pos,
        &building_q,
        tile_size.tile_size,
    );

    if met {
        let toast_msg = step.toast.clone();
        if step.persistent {
            toast_queue.push_persistent(toast_msg);
        } else {
            toast_queue.push(toast_msg);
        }
        state.current_index += 1;
        if state.current_index >= state.steps.len() {
            state.completed = true;
        }
    }

    // Allow skipping with Tab for debugging
    if keys.just_pressed(KeyCode::Tab) {
        let toast_msg = state.steps[state.current_index].toast.clone();
        toast_queue.push(toast_msg);
        state.current_index += 1;
        if state.current_index >= state.steps.len() {
            state.completed = true;
        }
    }
}

fn evaluate_condition(
    condition: &str,
    params: &HashMap<String, String>,
    conditions: &TutorialConditions,
    player_pos: &PlayerWorldPos,
    building_q: &Query<(&Building, &TilePosition)>,
    tile_size: f32,
) -> bool {
    match condition {
        "player_moved_distance" => {
            let required: f32 = params
                .get("distance")
                .and_then(|s| s.parse().ok())
                .unwrap_or(5.0);
            conditions.player_moved_distance >= required
        }
        "items_collected" => {
            let required: u32 = params
                .get("count")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            conditions.items_collected >= required
        }
        "item_crafted" => {
            if let Some(recipe_id) = params.get("recipe_id") {
                conditions
                    .items_crafted
                    .get(recipe_id)
                    .copied()
                    .unwrap_or(0)
                    >= 1
            } else {
                false
            }
        }
        "building_placed" => {
            if let Some(building_id) = params.get("building_id") {
                conditions
                    .building_types_placed
                    .get(building_id)
                    .copied()
                    .unwrap_or(0)
                    >= 1
            } else {
                conditions.buildings_placed >= 1
            }
        }
        "player_near_structure" => {
            let required_distance: f32 = params
                .get("distance")
                .and_then(|s| s.parse().ok())
                .unwrap_or(3.0);
            let structure_id = params.get("structure_id").map(|s| s.as_str());
            let (player_tile_x, player_tile_y) = world_to_tile(player_pos.0.truncate(), tile_size);
            for (building, tile_pos) in building_q.iter() {
                if let Some(sid) = structure_id
                    && building.kind != sid {
                        continue;
                    }
                let dx = (tile_pos.x - player_tile_x) as f32;
                let dy = (tile_pos.y - player_tile_y) as f32;
                if (dx * dx + dy * dy).sqrt() <= required_distance {
                    return true;
                }
            }
            false
        }
        "structure_interacted" => {
            let structure_id = params.get("structure_id").map(|s| s.as_str());
            if let Some(sid) = structure_id {
                conditions
                    .structures_interacted
                    .get(sid)
                    .copied()
                    .unwrap_or(false)
            } else {
                conditions.has_interacted_with_structure
            }
        }
        _ => {
            warn!("Unknown tutorial condition: {}", condition);
            true
        }
    }
}

#[derive(Resource, Default)]
pub struct TutorialHighlightEntity(pub Option<Entity>);

// SUGGEST: extraire dans un struct SystemParam (clippy::too_many_arguments)
pub fn tutorial_highlight_system(
    mut commands: Commands,
    state: Res<TutorialState>,
    mut highlight_entity: ResMut<TutorialHighlightEntity>,
    shapes: Res<ShapeCache>,
    config: Res<VisualsConfig>,
    cfg: Res<MapConfig>,
    building_q: Query<(&Building, &TilePosition)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if state.completed || state.steps.is_empty() {
        if let Some(entity) = highlight_entity.0.take() {
            silent_despawn(&mut commands, entity);
        }
        return;
    }

    let step = &state.steps[state.current_index];
    let Some(ref highlight) = step.highlight else {
        if let Some(entity) = highlight_entity.0.take() {
            silent_despawn(&mut commands, entity);
        }
        return;
    };

    let target_tile = match highlight.target_type.as_str() {
        "structure" => {
            let id = highlight.target_id.as_deref().unwrap_or("");
            building_q
                .iter()
                .find(|(b, _)| b.kind == id)
                .map(|(_, tp)| *tp)
        }
        "tile" => {
            let tx = highlight.tile_x.unwrap_or(0);
            let ty = highlight.tile_y.unwrap_or(0);
            Some(TilePosition { x: tx, y: ty })
        }
        _ => None,
    };

    let Some(tile) = target_tile else {
        if let Some(entity) = highlight_entity.0.take() {
            silent_despawn(&mut commands, entity);
        }
        return;
    };

    let world_pos = tile_to_world(tile.x, tile.y, cfg.tile_size);
    let world_x = world_pos.x;
    let world_y = world_pos.y;
    let z = config.tile_highlight.z + 0.1;

    if let Some(entity) = highlight_entity.0 {
        commands
            .entity(entity)
            .insert(Transform::from_xyz(world_x, world_y, z));
    } else {
        let entity = commands
            .spawn((
                Mesh2d(shapes.diamond.clone()),
                MeshMaterial2d(materials.add(Color::srgba(1.0, 0.8, 0.2, 0.4))),
                Transform::from_xyz(world_x, world_y, z),
            ))
            .id();
        highlight_entity.0 = Some(entity);
    }
}
