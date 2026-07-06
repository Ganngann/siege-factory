use crate::economy::resource::ResourceId;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RecipeDef {
    pub id: String,
    pub category: String,
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
        let toml_str = include_str!("../../data/recipes.toml");
        let parsed: RecipesToml = toml::from_str(toml_str).expect("failed to parse recipes.toml");
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
}

#[derive(Deserialize)]
struct RecipesToml {
    recipes: HashMap<String, RecipeEntry>,
}

#[derive(Deserialize)]
struct RecipeEntry {
    category: String,
    input: HashMap<String, u32>,
    output: HashMap<String, u32>,
    time_sec: f32,
}
