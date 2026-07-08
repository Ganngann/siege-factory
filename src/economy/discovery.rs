use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;

use crate::core::toast::ToastQueue;
use crate::economy::components::{Building, DiscoveredRecipes, ProductionCounter};
use crate::economy::resource::ResourceRegistry;
#[derive(Debug, Clone)]
pub struct DiscoveryDef {
    pub building: String,
    pub threshold: u32,
    pub reward_type: String,
    pub reward_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Resource)]
pub struct DiscoveryRegistry {
    pub discoveries: Vec<DiscoveryDef>,
    pub starter_recipes: Vec<String>,
}

impl DiscoveryRegistry {
    pub fn load(mods: &crate::core::modding::ModRegistry) -> Self {
        let mut discoveries = Vec::new();
        let mut starter_recipes = Vec::new();
        for (_mod_id, parsed) in mods.load_all_toml::<DiscoveriesToml>("discoveries.toml") {
            for entry in parsed.discovery {
                discoveries.push(DiscoveryDef {
                    building: entry.building,
                    threshold: entry.threshold,
                    reward_type: entry.reward_type,
                    reward_id: entry.id,
                    message: entry.message,
                });
            }
            if starter_recipes.is_empty() {
                starter_recipes = parsed.starter_recipes.recipes;
            }
        }
        Self {
            discoveries,
            starter_recipes,
        }
    }
}

#[derive(Deserialize)]
struct DiscoveriesToml {
    #[serde(rename = "discovery")]
    discovery: Vec<DiscoveryEntry>,
    #[serde(default)]
    starter_recipes: StarterRecipes,
}

#[derive(Default, Deserialize)]
struct StarterRecipes {
    #[serde(default)]
    recipes: Vec<String>,
}

#[derive(Deserialize)]
struct DiscoveryEntry {
    building: String,
    threshold: u32,
    #[serde(rename = "type")]
    reward_type: String,
    id: String,
    message: String,
}

#[derive(Debug, Clone, Resource)]
pub struct GlobalArchive {
    pub unlocked_recipes: HashSet<String>,
}

impl GlobalArchive {
    pub fn new(starter_recipes: &[String]) -> Self {
        Self {
            unlocked_recipes: starter_recipes.iter().cloned().collect(),
        }
    }

    pub fn is_unlocked(&self, recipe_id: &str) -> bool {
        self.unlocked_recipes.contains(recipe_id)
    }
}

#[derive(Event)]
pub struct DiscoveryEvent {
    pub building: Entity,
    pub discovery_id: String,
    pub message: String,
}

pub fn check_discoveries(
    registry: Res<DiscoveryRegistry>,
    global_archive: Res<GlobalArchive>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &ProductionCounter,
        &mut DiscoveredRecipes,
        &Building,
    )>,
) {
    for (entity, counter, mut discovered, building) in &mut query {
        for def in &registry.discoveries {
            if def.building != building.kind {
                continue;
            }
            if counter.0 < def.threshold {
                continue;
            }
            if discovered.0.iter().any(|id| id == &def.reward_id) {
                continue;
            }
            if global_archive.is_unlocked(&def.reward_id) {
                continue;
            }

            discovered.0.push(def.reward_id.clone());
            commands.trigger(DiscoveryEvent {
                building: entity,
                discovery_id: def.reward_id.clone(),
                message: def.message.clone(),
            });
        }
    }
}

pub fn on_discovery(
    on: On<DiscoveryEvent>,
    resource_registry: Res<ResourceRegistry>,
    building_query: Query<&Building>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    let event = on.event();
    let building_name = building_query
        .get(event.building)
        .map(|b| b.name.as_str())
        .unwrap_or("Building");
    let item_name = resource_registry
        .get_opt(&event.discovery_id)
        .map(|r| r.name.as_str())
        .unwrap_or(&event.discovery_id);
    toast_queue
        .0
        .push(format!("{}: {} discovered!", building_name, item_name));
    toast_queue
        .0
        .push("Craft it and bring it to the Archive!".to_string());
}



