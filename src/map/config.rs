use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Resource)]
pub struct MapConfig {
    pub tile_size: f32,
    pub seed: u64,
    pub chunk_size: u32,
    pub deposit_min_amount: u32,
    pub deposit_max_amount: u32,
    pub deposit_spawn_chance_pct: u32,
    pub deposit_min_per_chunk: u32,
    pub deposit_max_per_chunk: u32,
    pub deposit_distribution: Vec<(String, u32)>,
    pub infinite_deposits: bool,
    pub player_start_position: (i32, i32),
    pub player_hp: u32,
    pub player_speed: f32,
    pub builder_speed: f32,
    pub builder_reach: f32,
    pub pathfinding_max_nodes: usize,
}

impl MapConfig {
    pub fn load() -> Self {
        let toml_str = include_str!("../../data/map_config.toml");
        let parsed: MapToml = toml::from_str(toml_str).expect("failed to parse map_config.toml");
        let mut distribution: Vec<(String, u32)> =
            parsed.deposits.distribution.into_iter().collect();
        distribution.sort_by(|a, b| b.1.cmp(&a.1));
        Self {
            tile_size: parsed.map.tile_size,
            seed: parsed.map.seed,
            chunk_size: parsed.map.chunk_size,
            deposit_min_amount: parsed.deposits.min_amount,
            deposit_max_amount: parsed.deposits.max_amount,
            deposit_spawn_chance_pct: parsed.deposits.spawn_chance_pct,
            deposit_min_per_chunk: parsed.deposits.min_per_chunk,
            deposit_max_per_chunk: parsed.deposits.max_per_chunk,
            deposit_distribution: distribution,
            infinite_deposits: parsed.deposits.infinite,
            player_start_position: (parsed.player.position.x, parsed.player.position.y),
            player_hp: parsed.player.hp,
            player_speed: parsed.player.speed,
            builder_speed: parsed.player.builder_speed,
            builder_reach: parsed.player.builder_reach,
            pathfinding_max_nodes: parsed.map.pathfinding_max_nodes as usize,
        }
    }
}

#[derive(Deserialize)]
struct MapToml {
    map: MapEntry,
    deposits: DepositsEntry,
    player: PlayerEntry,
}

#[derive(Deserialize)]
struct MapEntry {
    tile_size: f32,
    seed: u64,
    chunk_size: u32,
    #[serde(default = "default_pathfinding_nodes")]
    pathfinding_max_nodes: u64,
}

fn default_pathfinding_nodes() -> u64 { 50000 }

#[derive(Deserialize)]
struct DepositsEntry {
    min_amount: u32,
    max_amount: u32,
    spawn_chance_pct: u32,
    min_per_chunk: u32,
    max_per_chunk: u32,
    infinite: bool,
    distribution: HashMap<String, u32>,
}

#[derive(Deserialize)]
struct PlayerEntry {
    hp: u32,
    #[serde(default = "default_player_speed")]
    speed: f32,
    #[serde(default = "default_builder_speed")]
    builder_speed: f32,
    #[serde(default = "default_builder_reach")]
    builder_reach: f32,
    position: PosEntry,
}

fn default_player_speed() -> f32 { 250.0 }
fn default_builder_speed() -> f32 { 300.0 }
fn default_builder_reach() -> f32 { 8.0 }

#[derive(Deserialize)]
struct PosEntry {
    x: i32,
    y: i32,
}
