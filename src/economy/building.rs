use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use crate::economy::resource::ResourceId;

fn parse_hex_color(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(255) as f32 / 255.0;
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(255) as f32 / 255.0;
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(255) as f32 / 255.0;
    Color::srgb(r, g, b)
}

#[derive(Debug, Clone)]
pub struct BuildingCost {
    pub resource: ResourceId,
    pub amount: u32,
}

#[derive(Debug, Clone)]
pub struct CombatStats {
    pub damage: u32,
    pub range: f32,
    pub fire_rate_sec: f32,
}

#[derive(Debug, Clone)]
pub struct BuildingDef {
    pub id: String,
    pub name: String,
    pub cost: Vec<BuildingCost>,
    pub hp: u32,
    pub tile_size: (u32, u32),
    pub color: Color,
    pub combat: Option<CombatStats>,
}

#[derive(Debug, Clone, Resource)]
pub struct BuildingRegistry {
    pub buildings: Vec<BuildingDef>,
}

impl BuildingRegistry {
    pub fn load() -> Self {
        let toml_str = include_str!("../../data/buildings.toml");
        let parsed: BuildingsToml = toml::from_str(toml_str).expect("failed to parse buildings.toml");
        let mut buildings = Vec::new();
        for (id, entry) in parsed.buildings {
            let mut cost = Vec::new();
            for (res_key, amount) in entry.cost {
                if let Some(resource) = ResourceId::from_str(&res_key) {
                    cost.push(BuildingCost { resource, amount });
                }
            }
            let color = entry.color.as_deref()
                .map(parse_hex_color)
                .unwrap_or(Color::srgb(0.5, 0.5, 0.5));
            let combat = entry.combat.map(|c| CombatStats {
                damage: c.damage,
                range: c.range * c.range, // square for distance² comparisons
                fire_rate_sec: c.fire_rate_sec,
            });
            buildings.push(BuildingDef {
                id: id.clone(),
                name: entry.name,
                cost,
                hp: entry.hp,
                tile_size: (entry.tile_size.w, entry.tile_size.h),
                color,
                combat,
            });
        }
        Self { buildings }
    }

    pub fn get(&self, id: &str) -> Option<&BuildingDef> {
        self.buildings.iter().find(|b| b.id == id)
    }
}

#[derive(Deserialize)]
struct BuildingsToml {
    buildings: HashMap<String, BuildingEntry>,
}

#[derive(Deserialize)]
struct BuildingEntry {
    name: String,
    cost: HashMap<String, u32>,
    hp: u32,
    tile_size: TileSize,
    color: Option<String>,
    #[serde(default)]
    combat: Option<CombatEntry>,
}

#[derive(Deserialize)]
struct TileSize {
    w: u32,
    h: u32,
}

#[derive(Deserialize)]
struct CombatEntry {
    damage: u32,
    range: f32,
    fire_rate_sec: f32,
}
