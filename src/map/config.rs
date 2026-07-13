#![allow(clippy::unnecessary_sort_by)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::type_complexity)]
#![allow(clippy::drop_non_drop)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::useless_format)]
#![allow(clippy::single_match)]
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
    pub fn load(mods: &crate::core::modding::ModRegistry) -> Self {
        let parsed: Vec<(String, MapToml)> = mods.load_all_toml("map_config.toml");
        let mut merged = MapToml::default();
        for (_mod_id, overlay) in parsed.iter() {
            let b_map = merged.map.as_ref().cloned().unwrap_or_default();
            let b_dep = merged.deposits.as_ref().cloned().unwrap_or_default();
            let b_plr = merged.player.as_ref().cloned().unwrap_or_default();
            if let Some(ref m) = overlay.map {
                merged.map = Some(merge_map_entry(&b_map, m));
            }
            if let Some(ref d) = overlay.deposits {
                merged.deposits = Some(merge_deposits(&b_dep, d));
            }
            if let Some(ref p) = overlay.player {
                merged.player = Some(merge_player(&b_plr, p));
            }
            if overlay.chunk.is_some() {
                merged.chunk = overlay.chunk.clone();
            }
            if overlay.decoration.is_some() {
                merged.decoration = overlay.decoration.clone();
            }
            if overlay.starting_area.is_some() {
                merged.starting_area = overlay.starting_area.clone();
            }
        }
        let map = merged.map.expect("MapConfig: missing required 'map' section");
        let deposits = merged.deposits.expect("MapConfig: missing required 'deposits' section");
        let player = merged.player.expect("MapConfig: missing required 'player' section");
        let chunk = merged.chunk.unwrap_or_default();
        let decoration = merged.decoration.unwrap_or_default();
        let mut distribution: Vec<(String, u32)> =
            deposits.distribution.into_iter().collect();
        // SUGGEST: utiliser sort_by_key(|b| std::cmp::Reverse(b.1)) (clippy::unnecessary_sort_by)
        distribution.sort_by(|a, b| b.1.cmp(&a.1));
        Self {
            tile_size: map.tile_size,
            seed: map.seed,
            chunk_size: map.chunk_size,
            deposit_min_amount: deposits.min_amount,
            deposit_max_amount: deposits.max_amount,
            deposit_spawn_chance_pct: deposits.spawn_chance_pct,
            deposit_min_per_chunk: deposits.min_per_chunk,
            deposit_max_per_chunk: deposits.max_per_chunk,
            deposit_distribution: distribution,
            infinite_deposits: deposits.infinite,
            resource_discovery_map: deposits.resource_discovery_map.unwrap_or_default(),
            player_start_position: (player.position.x, player.position.y),
            player_hp: player.hp,
            player_speed: player.speed,
            builder_speed: player.builder_speed,
            builder_reach: player.builder_reach,
            pathfinding_max_nodes: map.pathfinding_max_nodes as usize,
            initial_margin: chunk.initial_margin,
            despawn_margin: chunk.despawn_margin,
            inspect_range_tiles: player.inspect_range_tiles,
            builder_range_tiles: player.builder_range_tiles,
            builder_idle_offset_x: player.builder_idle_offset_x,
            builder_idle_offset_y: player.builder_idle_offset_y,
            decoration_min_count: decoration.min_count,
            decoration_count_variance: decoration.count_variance,
            player_mining_interval: player.mining_interval,
            starting_area: {
                let sa = merged.starting_area.unwrap_or_default();
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

fn merge_map_entry(_base: &MapEntry, overlay: &MapEntry) -> MapEntry {
    MapEntry {
        tile_size: if overlay.tile_size != 0.0 { overlay.tile_size } else { _base.tile_size },
        seed: if overlay.seed != 0 { overlay.seed } else { _base.seed },
        chunk_size: if overlay.chunk_size != 0 { overlay.chunk_size } else { _base.chunk_size },
        pathfinding_max_nodes: overlay.pathfinding_max_nodes,
    }
}

fn merge_deposits(_base: &DepositsEntry, overlay: &DepositsEntry) -> DepositsEntry {
    DepositsEntry {
        min_amount: if overlay.min_amount != 0 { overlay.min_amount } else { _base.min_amount },
        max_amount: if overlay.max_amount != 0 { overlay.max_amount } else { _base.max_amount },
        spawn_chance_pct: if overlay.spawn_chance_pct != 0 { overlay.spawn_chance_pct } else { _base.spawn_chance_pct },
        min_per_chunk: if overlay.min_per_chunk != 0 { overlay.min_per_chunk } else { _base.min_per_chunk },
        max_per_chunk: if overlay.max_per_chunk != 0 { overlay.max_per_chunk } else { _base.max_per_chunk },
        infinite: overlay.infinite,
        distribution: if !overlay.distribution.is_empty() { overlay.distribution.clone() } else { _base.distribution.clone() },
        resource_discovery_map: overlay.resource_discovery_map.clone().or_else(|| _base.resource_discovery_map.clone()),
    }
}

fn merge_player(_base: &PlayerEntry, overlay: &PlayerEntry) -> PlayerEntry {
    PlayerEntry {
        hp: if overlay.hp != 0 { overlay.hp } else { _base.hp },
        speed: if overlay.speed != 0.0 { overlay.speed } else { _base.speed },
        builder_speed: if overlay.builder_speed != 0.0 { overlay.builder_speed } else { _base.builder_speed },
        builder_reach: if overlay.builder_reach != 0.0 { overlay.builder_reach } else { _base.builder_reach },
        position: PosEntry {
            x: if overlay.position.x != 0 { overlay.position.x } else { _base.position.x },
            y: if overlay.position.y != 0 { overlay.position.y } else { _base.position.y },
        },
        inspect_range_tiles: if overlay.inspect_range_tiles != 0.0 { overlay.inspect_range_tiles } else { _base.inspect_range_tiles },
        builder_range_tiles: if overlay.builder_range_tiles != 0.0 { overlay.builder_range_tiles } else { _base.builder_range_tiles },
        builder_idle_offset_x: if overlay.builder_idle_offset_x != 0.0 { overlay.builder_idle_offset_x } else { _base.builder_idle_offset_x },
        builder_idle_offset_y: if overlay.builder_idle_offset_y != 0.0 { overlay.builder_idle_offset_y } else { _base.builder_idle_offset_y },
        mining_interval: if overlay.mining_interval != 0.0 { overlay.mining_interval } else { _base.mining_interval },
    }
}

#[derive(Default, Clone, Deserialize)]
struct MapToml {
    #[serde(default)]
    map: Option<MapEntry>,
    #[serde(default)]
    deposits: Option<DepositsEntry>,
    #[serde(default)]
    player: Option<PlayerEntry>,
    #[serde(default)]
    chunk: Option<ChunkEntry>,
    #[serde(default)]
    decoration: Option<DecorationEntry>,
    #[serde(default)]
    starting_area: Option<StartingAreaEntry>,
}

#[derive(Default, Clone, Deserialize)]
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

#[derive(Clone, Deserialize)]
struct PlacedStructureEntry {
    kind: String,
    tile_x: i32,
    tile_y: i32,
    #[serde(default)]
    props: PlacedStructurePropsEntry,
}

#[derive(Default, Clone, Deserialize)]
struct PlacedStructurePropsEntry {
    #[serde(default)]
    resource: Option<String>,
    #[serde(default)]
    amount: Option<u32>,
    #[serde(default)]
    decoration_kind: Option<String>,
}

#[derive(Default, Clone, Deserialize)]
struct MapEntry {
    #[serde(default)]
    tile_size: f32,
    #[serde(default)]
    seed: u64,
    #[serde(default)]
    chunk_size: u32,
    #[serde(default = "default_pathfinding_nodes")]
    pathfinding_max_nodes: u64,
}

fn default_pathfinding_nodes() -> u64 {
    50000
}

#[derive(Default, Clone, Deserialize)]
struct DepositsEntry {
    #[serde(default)]
    min_amount: u32,
    #[serde(default)]
    max_amount: u32,
    #[serde(default)]
    spawn_chance_pct: u32,
    #[serde(default)]
    min_per_chunk: u32,
    #[serde(default)]
    max_per_chunk: u32,
    #[serde(default)]
    infinite: bool,
    #[serde(default)]
    distribution: HashMap<String, u32>,
    #[serde(default)]
    resource_discovery_map: Option<HashMap<String, String>>,
}

#[derive(Default, Clone, Deserialize)]
struct PlayerEntry {
    #[serde(default)]
    hp: u32,
    #[serde(default = "default_player_speed")]
    speed: f32,
    #[serde(default = "default_builder_speed")]
    builder_speed: f32,
    #[serde(default = "default_builder_reach")]
    builder_reach: f32,
    #[serde(default)]
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

#[derive(Default, Clone, Deserialize)]
struct PosEntry {
    #[serde(default)]
    x: i32,
    #[serde(default)]
    y: i32,
}

#[derive(Default, Clone, Deserialize)]
struct ChunkEntry {
    #[serde(default = "default_initial_margin")]
    initial_margin: i32,
    #[serde(default = "default_despawn_margin")]
    despawn_margin: i32,
}

#[derive(Default, Clone, Deserialize)]
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
