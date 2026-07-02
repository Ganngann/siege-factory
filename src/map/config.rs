use bevy::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Resource)]
pub struct MapConfig {
    pub tile_size: f32,
    pub width: u32,
    pub height: u32,
    pub deposit_positions: Vec<(u32, u32)>,
    pub deposit_min_amount: u32,
    pub deposit_max_amount: u32,
    pub hq_position: (u32, u32),
    pub hq_start_ore: u32,
    pub hq_hp: u32,
}

impl MapConfig {
    pub fn load() -> Self {
        let toml_str = include_str!("../../data/map_config.toml");
        let parsed: MapToml = toml::from_str(toml_str).expect("failed to parse map_config.toml");
        let deposits: Vec<(u32, u32)> = parsed.deposits.positions.iter()
            .map(|p| (p.x, p.y))
            .collect();
        Self {
            tile_size: parsed.map.tile_size,
            width: parsed.map.width,
            height: parsed.map.height,
            deposit_positions: deposits,
            deposit_min_amount: parsed.deposits.min_amount,
            deposit_max_amount: parsed.deposits.max_amount,
            hq_position: (parsed.hq.position.x, parsed.hq.position.y),
            hq_start_ore: parsed.hq.start_ore,
            hq_hp: parsed.hq.hp,
        }
    }
}

#[derive(Deserialize)]
struct MapToml {
    map: MapEntry,
    deposits: DepositsEntry,
    hq: HqEntry,
}

#[derive(Deserialize)]
struct MapEntry {
    tile_size: f32,
    width: u32,
    height: u32,
}

#[derive(Deserialize)]
struct DepositsEntry {
    min_amount: u32,
    max_amount: u32,
    positions: Vec<PosEntry>,
}

#[derive(Deserialize)]
struct PosEntry {
    x: u32,
    y: u32,
}

#[derive(Deserialize)]
struct HqEntry {
    start_ore: u32,
    hp: u32,
    position: PosEntry,
}
