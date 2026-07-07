use crate::load_toml;
use bevy::prelude::*;
use serde::Deserialize;

use crate::core::utils::parse_hex_color;

#[derive(Debug, Clone, Resource)]
pub struct VisualsConfig {
    pub hp_bar: HpBarConfig,
    pub belt_item: BeltItemConfig,
    pub unit: UnitVisualConfig,
    pub player: PlayerVisualConfig,
    pub builder: BuilderVisualConfig,
    pub enemy: EnemyVisualConfig,
    pub projectile: ProjectileConfig,
    pub mining_bar: MiningBarConfig,
    pub toast: ToastConfig,
    pub tile_highlight: TileHighlightConfig,
    pub deposit_sprite: DepositSpriteConfig,
    pub decoration: DecorationConfig,
    pub chunk_colors: ChunkColorsConfig,
    pub ghost: GhostConfig,
    pub building: BuildingVisualConfig,
}

#[derive(Debug, Clone)]
pub struct HpBarConfig {
    pub width: f32,
    pub height: f32,
    pub y_offset: f32,
    pub z: f32,
    pub color_high: Color,
    pub color_mid: Color,
    pub color_low: Color,
}

#[derive(Debug, Clone)]
pub struct BeltItemConfig {
    pub width: f32,
    pub height: f32,
    pub z: f32,
}

#[derive(Debug, Clone)]
pub struct UnitVisualConfig {
    pub width: f32,
    pub height: f32,
    pub z: f32,
    pub owner_color: Color,
}

#[derive(Debug, Clone)]
pub struct PlayerVisualConfig {
    pub width: f32,
    pub height: f32,
    pub z: f32,
}

#[derive(Debug, Clone)]
pub struct BuilderVisualConfig {
    pub width: f32,
    pub height: f32,
    pub z: f32,
}

#[derive(Debug, Clone)]
pub struct EnemyVisualConfig {
    pub boss_size: f32,
    pub tank_size: f32,
    pub default_size: f32,
    pub spawn_z: f32,
}

#[derive(Debug, Clone)]
pub struct ProjectileConfig {
    pub scale: f32,
}

#[derive(Debug, Clone)]
pub struct MiningBarConfig {
    pub width: f32,
    pub height: f32,
    pub y_offset: f32,
    pub z: f32,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct ToastConfig {
    pub lifetime: f32,
    pub font_size: f32,
    pub color: Color,
    pub bottom_px: f32,
}

#[derive(Debug, Clone)]
pub struct TileHighlightConfig {
    pub color: Color,
    pub alpha: f32,
    pub z: f32,
}

#[derive(Debug, Clone)]
pub struct DepositSpriteConfig {
    pub scale_ratio: f32,
    pub z: f32,
    pub fallback_color: Color,
}

#[derive(Debug, Clone)]
pub struct DecorationConfig {
    pub tree_z: f32,
    pub rock_z: f32,
    pub tree_color: Color,
    pub rock_color: Color,
}

#[derive(Debug, Clone)]
pub struct ChunkColorsConfig {
    pub even: Color,
    pub odd: Color,
}

#[derive(Debug, Clone)]
pub struct GhostConfig {
    pub tint_r: f32,
    pub tint_g: f32,
    pub tint_b: f32,
    pub tint_a: f32,
}

#[derive(Debug, Clone)]
pub struct BuildingVisualConfig {
    pub owner_color: Color,
    pub level_color: Color,
}

impl VisualsConfig {
    pub fn load() -> Self {
        let parsed: VisualsToml = load_toml!("../../data/visuals.toml", VisualsToml);
        Self {
            hp_bar: HpBarConfig {
                width: parsed.hp_bar.width,
                height: parsed.hp_bar.height,
                y_offset: parsed.hp_bar.y_offset,
                z: parsed.hp_bar.z,
                color_high: parse_hex_color(&parsed.hp_bar.color_high),
                color_mid: parse_hex_color(&parsed.hp_bar.color_mid),
                color_low: parse_hex_color(&parsed.hp_bar.color_low),
            },
            belt_item: BeltItemConfig {
                width: parsed.belt_item.width,
                height: parsed.belt_item.height,
                z: parsed.belt_item.z,
            },
            unit: UnitVisualConfig {
                width: parsed.unit.width,
                height: parsed.unit.height,
                z: parsed.unit.z,
                owner_color: parse_hex_color(&parsed.unit.owner_color),
            },
            player: PlayerVisualConfig {
                width: parsed.player.width,
                height: parsed.player.height,
                z: parsed.player.z,
            },
            builder: BuilderVisualConfig {
                width: parsed.builder.width,
                height: parsed.builder.height,
                z: parsed.builder.z,
            },
            enemy: EnemyVisualConfig {
                boss_size: parsed.enemy.boss_size,
                tank_size: parsed.enemy.tank_size,
                default_size: parsed.enemy.default_size,
                spawn_z: parsed.enemy.spawn_z,
            },
            projectile: ProjectileConfig {
                scale: parsed.projectile.scale,
            },
            mining_bar: MiningBarConfig {
                width: parsed.mining_bar.width,
                height: parsed.mining_bar.height,
                y_offset: parsed.mining_bar.y_offset,
                z: parsed.mining_bar.z,
                color: parse_hex_color(&parsed.mining_bar.color),
            },
            toast: ToastConfig {
                lifetime: parsed.toast.lifetime,
                font_size: parsed.toast.font_size,
                color: parse_hex_color(&parsed.toast.color),
                bottom_px: parsed.toast.bottom_px,
            },
            tile_highlight: TileHighlightConfig {
                color: parse_hex_color(&parsed.tile_highlight.color),
                alpha: parsed.tile_highlight.alpha,
                z: parsed.tile_highlight.z,
            },
            deposit_sprite: DepositSpriteConfig {
                scale_ratio: parsed.deposit_sprite.scale_ratio,
                z: parsed.deposit_sprite.z,
                fallback_color: parse_hex_color(&parsed.deposit_sprite.fallback_color),
            },
            decoration: DecorationConfig {
                tree_z: parsed.decoration.tree_z,
                rock_z: parsed.decoration.rock_z,
                tree_color: parse_hex_color(&parsed.decoration.tree_color),
                rock_color: parse_hex_color(&parsed.decoration.rock_color),
            },
            chunk_colors: ChunkColorsConfig {
                even: parse_hex_color(&parsed.chunk_colors.even),
                odd: parse_hex_color(&parsed.chunk_colors.odd),
            },
            ghost: GhostConfig {
                tint_r: parsed.ghost.tint_r,
                tint_g: parsed.ghost.tint_g,
                tint_b: parsed.ghost.tint_b,
                tint_a: parsed.ghost.tint_a,
            },
            building: BuildingVisualConfig {
                owner_color: parse_hex_color(&parsed.building.owner_color),
                level_color: parse_hex_color(&parsed.building.level_color),
            },
        }
    }
}

#[derive(Deserialize)]
struct VisualsToml {
    hp_bar: HpBarEntry,
    belt_item: BeltItemEntry,
    unit: UnitEntry,
    player: PlayerEntry,
    builder: BuilderEntry,
    enemy: EnemyEntry,
    projectile: ProjectileEntry,
    mining_bar: MiningBarEntry,
    toast: ToastEntry,
    tile_highlight: TileHighlightEntry,
    deposit_sprite: DepositSpriteEntry,
    decoration: DecorationEntry,
    chunk_colors: ChunkColorsEntry,
    ghost: GhostEntry,
    building: BuildingEntry,
}

#[derive(Deserialize)]
struct HpBarEntry {
    width: f32,
    height: f32,
    y_offset: f32,
    z: f32,
    color_high: String,
    color_mid: String,
    color_low: String,
}
#[derive(Deserialize)]
struct BeltItemEntry {
    width: f32,
    height: f32,
    z: f32,
}
#[derive(Deserialize)]
struct UnitEntry {
    width: f32,
    height: f32,
    z: f32,
    owner_color: String,
}
#[derive(Deserialize)]
struct PlayerEntry {
    width: f32,
    height: f32,
    z: f32,
}
#[derive(Deserialize)]
struct BuilderEntry {
    width: f32,
    height: f32,
    z: f32,
}
#[derive(Deserialize)]
struct EnemyEntry {
    boss_size: f32,
    tank_size: f32,
    default_size: f32,
    spawn_z: f32,
}
#[derive(Deserialize)]
struct ProjectileEntry {
    scale: f32,
}
#[derive(Deserialize)]
struct MiningBarEntry {
    width: f32,
    height: f32,
    y_offset: f32,
    z: f32,
    color: String,
}
#[derive(Deserialize)]
struct ToastEntry {
    lifetime: f32,
    font_size: f32,
    color: String,
    bottom_px: f32,
}
#[derive(Deserialize)]
struct TileHighlightEntry {
    color: String,
    alpha: f32,
    z: f32,
}
#[derive(Deserialize)]
struct DepositSpriteEntry {
    scale_ratio: f32,
    z: f32,
    fallback_color: String,
}
#[derive(Deserialize)]
struct DecorationEntry {
    tree_z: f32,
    rock_z: f32,
    tree_color: String,
    rock_color: String,
}
#[derive(Deserialize)]
struct ChunkColorsEntry {
    even: String,
    odd: String,
}
#[derive(Deserialize)]
struct GhostEntry {
    tint_r: f32,
    tint_g: f32,
    tint_b: f32,
    tint_a: f32,
}
#[derive(Deserialize)]
struct BuildingEntry {
    owner_color: String,
    level_color: String,
}
