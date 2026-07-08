use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ToolDef {
    pub id: String,
    pub allowed_resources: Vec<String>,
    pub mine_interval_mult: f32,
}

#[derive(Debug, Clone, Resource)]
pub struct ToolRegistry {
    pub tools: HashMap<String, ToolDef>,
}

impl ToolRegistry {
    pub fn load(mods: &crate::core::modding::ModRegistry) -> Self {
        let mut tools = HashMap::new();
        for (_mod_id, parsed) in mods.load_all_toml::<ToolsToml>("tools.toml") {
            for (id, entry) in parsed.tools {
                tools.insert(
                    id.clone(),
                    ToolDef {
                        id,
                        allowed_resources: entry.allowed_resources,
                        mine_interval_mult: entry.mine_interval_mult,
                    },
                );
            }
        }
        Self { tools }
    }

    pub fn get(&self, id: &str) -> Option<&ToolDef> {
        self.tools.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.tools.contains_key(id)
    }

    /// Check if an item ID corresponds to a known tool that covers a given resource.
    pub fn tool_covers_resource(&self, tool_id: &str, resource: &str) -> bool {
        self.tools
            .get(tool_id)
            .map(|t| t.allowed_resources.iter().any(|r| r == resource))
            .unwrap_or(false)
    }

    /// Find the best tool multiplier for mining a given resource from the player's inventory.
    /// Returns (tool_id, mine_interval_mult) for the first matching tool found.
    pub fn best_tool_for<'a>(
        &'a self,
        resource: &str,
        inventory: &crate::economy::resource::Inventory,
    ) -> Option<(&'a str, f32)> {
        for slot in inventory.slots.iter() {
            if let Some((res_id, _amount)) = slot
                && let Some(def) = self.tools.get(&res_id.0)
                    && def.allowed_resources.iter().any(|r| r == resource) {
                        return Some((&def.id, def.mine_interval_mult));
                    }
        }
        None
    }
}

#[derive(Deserialize)]
struct ToolsToml {
    #[serde(default)]
    tools: HashMap<String, ToolEntry>,
}

#[derive(Deserialize)]
struct ToolEntry {
    #[serde(default)]
    allowed_resources: Vec<String>,
    #[serde(default = "default_mult")]
    mine_interval_mult: f32,
}

fn default_mult() -> f32 {
    0.5
}
