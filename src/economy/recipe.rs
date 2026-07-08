use crate::economy::resource::ResourceId;
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
    pub fn load(mods: &crate::core::modding::ModRegistry) -> Self {
        let mut recipes = HashMap::new();
        for (_mod_id, parsed) in mods.load_all_toml::<RecipesToml>("recipes.toml") {
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
        }
        Self { recipes }
    }

    pub fn get(&self, id: &str) -> Option<&RecipeDef> {
        self.recipes.get(id)
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



