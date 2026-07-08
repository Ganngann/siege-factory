use crate::economy::resource::ResourceId;
use crate::load_toml;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RecipeDef {
    pub id: String,
    pub category: String,
    pub craftable_in: Vec<String>,
    pub input: Vec<(ResourceId, u32)>,
    pub output: Vec<(ResourceId, u32)>,
    pub time_sec: f32,
}

#[derive(Debug, Clone, Resource)]
pub struct RecipeRegistry {
    pub recipes: HashMap<String, RecipeDef>,
}

impl RecipeRegistry {
    pub fn load() -> Self {
        let parsed: RecipesToml = load_toml!("../../data/recipes.toml", RecipesToml);
        let mut recipes = HashMap::new();
        for (id, entry) in parsed.recipes {
            let input = entry
                .input
                .iter()
                .map(|(k, v)| (ResourceId::new(k), *v))
                .collect();
            let output = entry
                .output
                .iter()
                .map(|(k, v)| (ResourceId::new(k), *v))
                .collect();
            recipes.insert(
                id.clone(),
                RecipeDef {
                    id,
                    category: entry.category,
                    craftable_in: entry.craftable_in,
                    input,
                    output,
                    time_sec: entry.time_sec,
                },
            );
        }
        Self { recipes }
    }

    pub fn get(&self, id: &str) -> Option<&RecipeDef> {
        self.recipes.get(id)
    }

    pub fn apply_mod_overrides(&mut self, mods: &crate::core::modding::ModRegistry) {
        let Some(content) = mods.load_data("recipes.toml") else {
            return;
        };
        let Ok(parsed) = toml::from_str::<RecipesToml>(&content) else {
            bevy::prelude::error!("Failed to parse recipes.toml from mod");
            return;
        };
        for (id, entry) in parsed.recipes {
            let input = entry
                .input
                .iter()
                .map(|(k, v)| (ResourceId::new(k), *v))
                .collect();
            let output = entry
                .output
                .iter()
                .map(|(k, v)| (ResourceId::new(k), *v))
                .collect();
            self.recipes.insert(
                id.clone(),
                RecipeDef {
                    id,
                    category: entry.category,
                    craftable_in: entry.craftable_in,
                    input,
                    output,
                    time_sec: entry.time_sec,
                },
            );
        }
    }
}

#[derive(Deserialize)]
struct RecipesToml {
    recipes: HashMap<String, RecipeEntry>,
}

#[derive(Deserialize)]
struct RecipeEntry {
    category: String,
    #[serde(default)]
    craftable_in: Vec<String>,
    input: HashMap<String, u32>,
    output: HashMap<String, u32>,
    time_sec: f32,
}


