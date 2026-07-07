use crate::load_toml;
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
    pub resource_discovery_map: HashMap<String, String>,
    pub player_start_position: (i32, i32),
    pub player_hp: u32,
    pub player_speed: f32,
    pub builder_speed: f32,
    pub builder_reach: f32,
    pub pathfinding_max_nodes: usize,
    pub initial_margin: i32,
    pub despawn_margin: i32,
    pub inspect_range_tiles: f32,
    pub builder_range_tiles: f32,
    pub builder_idle_offset_x: f32,
    pub builder_idle_offset_y: f32,
    pub decoration_min_count: u32,
    pub decoration_count_variance: u32,
    pub player_mining_interval: f32,
    pub starting_area: StartingAreaConfig,
}

#[derive(Debug, Clone)]
pub struct StartingAreaConfig {
    pub enable: bool,
    pub radius: u32,
    pub clear_trees: bool,
    pub structures: Vec<PlacedStructure>,
}

#[derive(Debug, Clone)]
pub struct PlacedStructure {
    pub kind: String,
    pub tile_x: i32,
    pub tile_y: i32,
    pub props: PlacedStructureProps,
}

#[derive(Debug, Clone, Default)]
pub struct PlacedStructureProps {
    pub resource: Option<String>,
    pub amount: Option<u32>,
    pub decoration_kind: Option<String>,
}

impl MapConfig {
    pub fn load() -> Self {
        let parsed: MapToml = load_toml!("../../data/map_config.toml", MapToml);
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
            resource_discovery_map: parsed.deposits.resource_discovery_map.unwrap_or_default(),
            player_start_position: (parsed.player.position.x, parsed.player.position.y),
            player_hp: parsed.player.hp,
            player_speed: parsed.player.speed,
            builder_speed: parsed.player.builder_speed,
            builder_reach: parsed.player.builder_reach,
            pathfinding_max_nodes: parsed.map.pathfinding_max_nodes as usize,
            initial_margin: parsed.chunk.initial_margin,
            despawn_margin: parsed.chunk.despawn_margin,
            inspect_range_tiles: parsed.player.inspect_range_tiles,
            builder_range_tiles: parsed.player.builder_range_tiles,
            builder_idle_offset_x: parsed.player.builder_idle_offset_x,
            builder_idle_offset_y: parsed.player.builder_idle_offset_y,
            decoration_min_count: parsed.decoration.min_count,
            decoration_count_variance: parsed.decoration.count_variance,
            player_mining_interval: parsed.player.mining_interval,
            starting_area: {
                let sa = parsed.starting_area.unwrap_or_default();
                StartingAreaConfig {
                    enable: sa.enable,
                    radius: sa.radius,
                    clear_trees: sa.clear_trees,
                    structures: sa
                        .structures
                        .unwrap_or_default()
                        .into_iter()
                        .map(|s| PlacedStructure {
                            kind: s.kind,
                            tile_x: s.tile_x,
                            tile_y: s.tile_y,
                            props: PlacedStructureProps {
                                resource: s.props.resource,
                                amount: s.props.amount,
                                decoration_kind: s.props.decoration_kind,
                            },
                        })
                        .collect(),
                }
            },
        }
    }
}

#[derive(Deserialize)]
struct MapToml {
    map: MapEntry,
    deposits: DepositsEntry,
    player: PlayerEntry,
    #[serde(default)]
    chunk: ChunkEntry,
    #[serde(default)]
    decoration: DecorationEntry,
    #[serde(default)]
    starting_area: Option<StartingAreaEntry>,
}

#[derive(Default, Deserialize)]
struct StartingAreaEntry {
    #[serde(default)]
    enable: bool,
    #[serde(default = "default_starting_radius")]
    radius: u32,
    #[serde(default)]
    clear_trees: bool,
    #[serde(default)]
    structures: Option<Vec<PlacedStructureEntry>>,
}

fn default_starting_radius() -> u32 {
    8
}

#[derive(Deserialize)]
struct PlacedStructureEntry {
    kind: String,
    tile_x: i32,
    tile_y: i32,
    #[serde(default)]
    props: PlacedStructurePropsEntry,
}

#[derive(Default, Deserialize)]
struct PlacedStructurePropsEntry {
    #[serde(default)]
    resource: Option<String>,
    #[serde(default)]
    amount: Option<u32>,
    #[serde(default)]
    decoration_kind: Option<String>,
}

#[derive(Deserialize)]
struct MapEntry {
    tile_size: f32,
    seed: u64,
    chunk_size: u32,
    #[serde(default = "default_pathfinding_nodes")]
    pathfinding_max_nodes: u64,
}

fn default_pathfinding_nodes() -> u64 {
    50000
}

#[derive(Deserialize)]
struct DepositsEntry {
    min_amount: u32,
    max_amount: u32,
    spawn_chance_pct: u32,
    min_per_chunk: u32,
    max_per_chunk: u32,
    infinite: bool,
    distribution: HashMap<String, u32>,
    #[serde(default)]
    resource_discovery_map: Option<HashMap<String, String>>,
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
    #[serde(default = "default_inspect_range_tiles")]
    inspect_range_tiles: f32,
    #[serde(default = "default_builder_range_tiles")]
    builder_range_tiles: f32,
    #[serde(default = "default_builder_idle_offset_x")]
    builder_idle_offset_x: f32,
    #[serde(default = "default_builder_idle_offset_y")]
    builder_idle_offset_y: f32,
    #[serde(default = "default_mining_interval")]
    mining_interval: f32,
}

fn default_player_speed() -> f32 {
    250.0
}
fn default_builder_speed() -> f32 {
    300.0
}
fn default_builder_reach() -> f32 {
    8.0
}
fn default_inspect_range_tiles() -> f32 {
    3.0
}
fn default_builder_range_tiles() -> f32 {
    5.0
}
fn default_builder_idle_offset_x() -> f32 {
    -24.0
}
fn default_builder_idle_offset_y() -> f32 {
    -24.0
}
fn default_mining_interval() -> f32 {
    1.0
}

#[derive(Deserialize)]
struct PosEntry {
    x: i32,
    y: i32,
}

#[derive(Default, Deserialize)]
struct ChunkEntry {
    #[serde(default = "default_initial_margin")]
    initial_margin: i32,
    #[serde(default = "default_despawn_margin")]
    despawn_margin: i32,
}

#[derive(Default, Deserialize)]
struct DecorationEntry {
    #[serde(default = "default_decoration_min_count")]
    min_count: u32,
    #[serde(default = "default_decoration_count_variance")]
    count_variance: u32,
}

fn default_initial_margin() -> i32 {
    15
}
fn default_despawn_margin() -> i32 {
    3
}
fn default_decoration_min_count() -> u32 {
    4
}
fn default_decoration_count_variance() -> u32 {
    5
}
