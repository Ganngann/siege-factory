use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use crate::economy::resource::ResourceId;

#[derive(Debug, Clone)]
pub struct RecipeDef {
    pub input_resource: ResourceId,
    pub input_amount: u32,
    pub output_resource: ResourceId,
    pub output_amount: u32,
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
            let input_res = entry.input.keys().next()
                .and_then(|k| ResourceId::from_str(k));
            let input_amount = entry.input.values().next().copied().unwrap_or(0);
            let output_res = entry.output.keys().next()
                .and_then(|k| ResourceId::from_str(k));
            let output_amount = entry.output.values().next().copied().unwrap_or(0);
            if let (Some(input_resource), Some(output_resource)) = (input_res, output_res) {
                recipes.insert(id, RecipeDef {
                    input_resource,
                    input_amount,
                    output_resource,
                    output_amount,
                    time_sec: entry.time_sec,
                });
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
    input: HashMap<String, u32>,
    output: HashMap<String, u32>,
    time_sec: f32,
}
