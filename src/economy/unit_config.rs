use crate::economy::resource::{Cost, ResourceId};
use crate::load_toml;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct UnitDef {
    pub id: String,
    pub name: String,
    pub cost: Vec<Cost>,
    pub hp: u32,
    pub color: Color,
    pub visual: String,
    pub texture_stem: String,
    pub kind: String,
    pub damage: u32,
    pub range_tiles: f32,
    pub fire_rate_sec: f32,
    pub projectile_speed: f32,
    pub speed: f32,
    pub mine_interval_sec: f32,
    pub carry_capacity: u32,
    pub spawn_offset_x: f32,
    pub spawn_offset_y: f32,
    pub spawn_offset_z: f32,
    pub projectile_color: Color,
}

#[derive(Debug, Clone, Resource)]
pub struct UnitConfig {
    pub units: HashMap<String, UnitDef>,
}

use crate::core::utils::parse_hex_color;

fn parse_cost(cost: &HashMap<String, u32>) -> Vec<Cost> {
    let mut result = Vec::new();
    for (key, amount) in cost {
        result.push(Cost {
            resource: ResourceId::new(key),
            amount: *amount,
        });
    }
    result
}

impl UnitConfig {
    pub fn load() -> Self {
        let parsed: HashMap<String, UnitEntry> =
            load_toml!("../../data/units.toml", HashMap<String, UnitEntry>);
        let mut units = HashMap::new();
        for (id, entry) in parsed {
            let def = UnitDef {
                id: id.clone(),
                name: entry.name,
                cost: parse_cost(&entry.cost),
                hp: entry.hp,
                color: parse_hex_color(&entry.color),
                visual: entry.visual.unwrap_or_else(|| "circle".to_string()),
                texture_stem: entry.texture_stem.unwrap_or_else(|| id.clone()),
                kind: entry.kind.unwrap_or_else(|| "combat".to_string()),
                damage: entry.damage.unwrap_or(0),
                range_tiles: entry.range_tiles.unwrap_or(0.0),
                fire_rate_sec: entry.fire_rate_sec.unwrap_or(0.0),
                projectile_speed: entry.projectile_speed.unwrap_or(300.0),
                speed: entry.speed.unwrap_or(0.0),
                mine_interval_sec: entry.mine_interval_sec.unwrap_or(0.0),
                carry_capacity: entry.carry_capacity.unwrap_or(5),
                spawn_offset_x: entry.spawn_offset_x.unwrap_or(0.0),
                spawn_offset_y: entry.spawn_offset_y.unwrap_or(0.0),
                spawn_offset_z: entry.spawn_offset_z.unwrap_or(2.5),
                projectile_color: entry
                    .projectile_color
                    .as_deref()
                    .map(parse_hex_color)
                    .unwrap_or(Color::srgb(0.3, 1.0, 0.3)),
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
    texture_stem: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    damage: Option<u32>,
    #[serde(default)]
    range_tiles: Option<f32>,
    #[serde(default)]
    fire_rate_sec: Option<f32>,
    #[serde(default)]
    projectile_speed: Option<f32>,
    #[serde(default)]
    speed: Option<f32>,
    #[serde(default)]
    mine_interval_sec: Option<f32>,
    #[serde(default)]
    carry_capacity: Option<u32>,
    #[serde(default)]
    spawn_offset_x: Option<f32>,
    #[serde(default)]
    spawn_offset_y: Option<f32>,
    #[serde(default)]
    spawn_offset_z: Option<f32>,
    #[serde(default)]
    projectile_color: Option<String>,
}
