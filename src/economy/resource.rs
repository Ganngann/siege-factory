use crate::core::utils::parse_hex_color;
use crate::load_toml;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub String);

impl ResourceId {
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(id.into().to_lowercase())
    }

    pub fn display_name(&self) -> String {
        self.0
            .split('_')
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().to_string() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cost {
    pub resource: ResourceId,
    pub amount: u32,
}

#[derive(Debug, Clone)]
pub struct ResourceDef {
    pub id: String,
    pub name: String,
    pub max_stack: u32,
    pub color: Color,
}

#[derive(Debug, Clone, Resource)]
pub struct ResourceRegistry {
    pub resources: HashMap<String, ResourceDef>,
}

impl ResourceRegistry {
    pub fn load() -> Self {
        let parsed: ResourcesToml = load_toml!("../../data/resources.toml", ResourcesToml);
        let mut resources = HashMap::new();
        for (key, entry) in parsed.resources {
            resources.insert(
                key.clone(),
                ResourceDef {
                    id: key,
                    name: entry.name,
                    max_stack: entry.max_stack,
                    color: entry
                        .color
                        .as_deref()
                        .map(parse_hex_color)
                        .unwrap_or(Color::srgb(0.5, 0.5, 0.5)),
                },
            );
        }
        Self { resources }
    }

    pub fn get(&self, id: &str) -> &ResourceDef {
        &self.resources[id]
    }

    pub fn get_opt(&self, id: &str) -> Option<&ResourceDef> {
        self.resources.get(id)
    }

    pub fn display_name<'a>(&'a self, id: &'a ResourceId) -> &'a str {
        match self.resources.get(&id.0) {
            Some(r) => &r.name,
            None => &id.0,
        }
    }

    pub fn apply_mod_overrides(&mut self, mods: &crate::core::modding::ModRegistry) {
        let Some(content) = mods.load_data("resources.toml") else {
            return;
        };
        let Ok(parsed) = toml::from_str::<ResourcesToml>(&content) else {
            bevy::prelude::error!("Failed to parse resources.toml from mod");
            return;
        };
        for (id, entry) in parsed.resources {
            self.resources.insert(
                id.clone(),
                ResourceDef {
                    id,
                    name: entry.name,
                    max_stack: entry.max_stack,
                    color: entry
                        .color
                        .as_deref()
                        .map(parse_hex_color)
                        .unwrap_or(Color::srgb(0.5, 0.5, 0.5)),
                },
            );
        }
    }
}

#[derive(Deserialize)]
struct ResourcesToml {
    resources: HashMap<String, ResourceEntry>,
}

#[derive(Deserialize)]
struct ResourceEntry {
    name: String,
    max_stack: u32,
    color: Option<String>,
}

#[derive(Debug, Clone, Component)]
pub struct Inventory {
    pub resources: HashMap<ResourceId, u32>,
    pub capacity: u32,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            capacity: 0,
        }
    }

    pub fn with_capacity(capacity: u32) -> Self {
        Self {
            resources: HashMap::new(),
            capacity,
        }
    }

    pub fn get(&self, resource: &ResourceId) -> u32 {
        *self.resources.get(resource).unwrap_or(&0)
    }

    pub fn add(&mut self, resource: &ResourceId, amount: u32) {
        let entry = self.resources.entry(resource.clone()).or_insert(0);
        *entry = entry.saturating_add(amount);
    }

    pub fn try_add(&mut self, resource: &ResourceId, amount: u32) -> bool {
        if self.capacity > 0 && self.total() + amount > self.capacity {
            return false;
        }
        self.add(resource, amount);
        true
    }

    pub fn remove(&mut self, resource: &ResourceId, amount: u32) -> bool {
        let entry = self.resources.entry(resource.clone()).or_insert(0);
        if *entry >= amount {
            *entry -= amount;
            true
        } else {
            false
        }
    }

    pub fn total(&self) -> u32 {
        self.resources.values().sum()
    }

    pub fn is_full(&self) -> bool {
        self.capacity > 0 && self.total() >= self.capacity
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}


