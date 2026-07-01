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
}

#[derive(Debug, Clone)]
pub struct SoldierDef {
    pub unit: UnitDef,
    pub damage: u32,
    pub range_tiles: f32,
    pub fire_rate_sec: f32,
}

#[derive(Debug, Clone)]
pub struct WorkerDef {
    pub unit: UnitDef,
    pub speed: f32,
    pub mine_interval_sec: f32,
}

#[derive(Debug, Clone, Resource)]
pub struct UnitConfig {
    pub soldier: SoldierDef,
    pub worker: WorkerDef,
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
        let parsed: UnitsToml = toml::from_str(toml_str).expect("failed to parse units.toml");
        Self {
            soldier: SoldierDef {
                damage: parsed.soldier.damage,
                range_tiles: parsed.soldier.range_tiles,
                fire_rate_sec: parsed.soldier.fire_rate_sec,
                unit: UnitDef {
                    id: "soldier".to_string(),
                    name: parsed.soldier.name,
                    cost: parse_cost(&parsed.soldier.cost),
                    hp: parsed.soldier.hp,
                    color: parse_hex_color(&parsed.soldier.color),
                },
            },
            worker: WorkerDef {
                speed: parsed.worker.speed,
                mine_interval_sec: parsed.worker.mine_interval_sec,
                unit: UnitDef {
                    id: "worker".to_string(),
                    name: parsed.worker.name,
                    cost: parse_cost(&parsed.worker.cost),
                    hp: parsed.worker.hp,
                    color: parse_hex_color(&parsed.worker.color),
                },
            },
        }
    }
}

#[derive(Deserialize)]
struct UnitsToml {
    soldier: SoldierEntry,
    worker: WorkerEntry,
}

#[derive(Deserialize)]
struct SoldierEntry {
    name: String,
    cost: HashMap<String, u32>,
    hp: u32,
    damage: u32,
    range_tiles: f32,
    fire_rate_sec: f32,
    color: String,
}

#[derive(Deserialize)]
struct WorkerEntry {
    name: String,
    cost: HashMap<String, u32>,
    hp: u32,
    speed: f32,
    mine_interval_sec: f32,
    color: String,
}
