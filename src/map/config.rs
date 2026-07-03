use bevy::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Resource)]
pub struct MapConfig {
    pub tile_size: f32,
    pub seed: u64,
    pub chunk_size: u32,
    pub deposit_min_amount: u32,
    pub deposit_max_amount: u32,
    pub hq_position: (i32, i32),
    pub hq_start_ore: u32,
    pub hq_hp: u32,
}

impl MapConfig {
    pub fn load() -> Self {
        let toml_str = include_str!("../../data/map_config.toml");
        let parsed: MapToml = toml::from_str(toml_str).expect("failed to parse map_config.toml");
        Self {
            tile_size: parsed.map.tile_size,
            seed: parsed.map.seed,
            chunk_size: parsed.map.chunk_size,
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
    seed: u64,
    chunk_size: u32,
}

#[derive(Deserialize)]
struct DepositsEntry {
    min_amount: u32,
    max_amount: u32,
}

#[derive(Deserialize)]
struct HqEntry {
    start_ore: u32,
    hp: u32,
    position: PosEntry,
}

#[derive(Deserialize)]
struct PosEntry {
    x: i32,
    y: i32,
}
