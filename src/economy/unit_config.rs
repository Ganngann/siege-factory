use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use crate::economy::resource::ResourceId;

#[derive(Debug, Clone)]
pub struct UnitCost {
    pub resource: ResourceId,
    pub amount: u32,
}

#[derive(Debug, Clone)]
pub struct UnitDef {
    pub id: String,
    pub name: String,
    pub cost: Vec<UnitCost>,
    pub hp: u32,
    pub color: Color,
    pub visual: String,
    pub kind: String,
    pub damage: u32,
    pub range_tiles: f32,
    pub fire_rate_sec: f32,
    pub speed: f32,
    pub mine_interval_sec: f32,
}

#[derive(Debug, Clone, Resource)]
pub struct UnitConfig {
    pub units: HashMap<String, UnitDef>,
}

fn parse_hex_color(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(255) as f32 / 255.0;
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(255) as f32 / 255.0;
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(255) as f32 / 255.0;
    Color::srgb(r, g, b)
}

fn parse_cost(cost: &HashMap<String, u32>) -> Vec<UnitCost> {
    let mut result = Vec::new();
    for (key, amount) in cost {
        if let Some(resource) = ResourceId::from_str(key) {
            result.push(UnitCost { resource, amount: *amount });
        }
    }
    result
}

impl UnitConfig {
    pub fn load() -> Self {
        let toml_str = include_str!("../../data/units.toml");
        let parsed: HashMap<String, UnitEntry> = toml::from_str(toml_str)
            .expect("failed to parse units.toml");
        let mut units = HashMap::new();
        for (id, entry) in parsed {
            let def = UnitDef {
                id: id.clone(),
                name: entry.name,
                cost: parse_cost(&entry.cost),
                hp: entry.hp,
                color: parse_hex_color(&entry.color),
                visual: entry.visual.unwrap_or_else(|| "circle".to_string()),
                kind: entry.kind.unwrap_or_else(|| "combat".to_string()),
                damage: entry.damage.unwrap_or(0),
                range_tiles: entry.range_tiles.unwrap_or(0.0),
                fire_rate_sec: entry.fire_rate_sec.unwrap_or(0.0),
                speed: entry.speed.unwrap_or(0.0),
                mine_interval_sec: entry.mine_interval_sec.unwrap_or(0.0),
            };
            units.insert(id, def);
        }
        Self { units }
    }

    pub fn get(&self, id: &str) -> Option<&UnitDef> {
        self.units.get(id)
    }
}

#[derive(Deserialize)]
struct UnitEntry {
    name: String,
    #[serde(default)]
    cost: HashMap<String, u32>,
    hp: u32,
    color: String,
    #[serde(default)]
    visual: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    damage: Option<u32>,
    #[serde(default)]
    range_tiles: Option<f32>,
    #[serde(default)]
    fire_rate_sec: Option<f32>,
    #[serde(default)]
    speed: Option<f32>,
    #[serde(default)]
    mine_interval_sec: Option<f32>,
}
