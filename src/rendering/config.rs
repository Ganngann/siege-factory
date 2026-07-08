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
    pub decorations: Vec<DecorationTypeConfig>,
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
pub struct DecorationTypeConfig {
    pub kind: String,
    pub min_size: f32,
    pub max_size: f32,
    pub scatter: f32,
    pub density: f32,
    pub z: f32,
    pub color: Color,
    pub shape: String,
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
    pub fn load(mods: &crate::core::modding::ModRegistry) -> Self {
        let all: Vec<VisualsToml> = mods
            .load_all_toml::<VisualsToml>("visuals.toml")
            .into_iter()
            .map(|(_, p)| p)
            .collect();
        let base = all.first().cloned().unwrap_or_else(|| panic!("No mod provides data/visuals.toml"));
        let mut decorations: Vec<DecorationTypeConfig> = all
            .into_iter()
            .flat_map(|p| p.decorations)
            .map(|d| DecorationTypeConfig {
                kind: d.kind,
                min_size: d.min_size,
                max_size: d.max_size,
                scatter: d.scatter,
                density: d.density,
                z: d.z,
                color: parse_hex_color(&d.color),
                shape: d.shape,
            })
            .collect();
        // Deduplicate by (kind, color)
        decorations.sort_by(|a, b| a.kind.cmp(&b.kind));
        decorations.dedup_by(|a, b| a.kind == b.kind);
        Self {
            hp_bar: HpBarConfig {
                width: base.hp_bar.width,
                height: base.hp_bar.height,
                y_offset: base.hp_bar.y_offset,
                z: base.hp_bar.z,
                color_high: parse_hex_color(&base.hp_bar.color_high),
                color_mid: parse_hex_color(&base.hp_bar.color_mid),
                color_low: parse_hex_color(&base.hp_bar.color_low),
            },
            belt_item: BeltItemConfig {
                width: base.belt_item.width,
                height: base.belt_item.height,
                z: base.belt_item.z,
            },
            unit: UnitVisualConfig {
                width: base.unit.width,
                height: base.unit.height,
                z: base.unit.z,
                owner_color: parse_hex_color(&base.unit.owner_color),
            },
            player: PlayerVisualConfig {
                width: base.player.width,
                height: base.player.height,
                z: base.player.z,
            },
            builder: BuilderVisualConfig {
                width: base.builder.width,
                height: base.builder.height,
                z: base.builder.z,
            },
            enemy: EnemyVisualConfig {
                boss_size: base.enemy.boss_size,
                tank_size: base.enemy.tank_size,
                default_size: base.enemy.default_size,
                spawn_z: base.enemy.spawn_z,
            },
            projectile: ProjectileConfig {
                scale: base.projectile.scale,
            },
            mining_bar: MiningBarConfig {
                width: base.mining_bar.width,
                height: base.mining_bar.height,
                y_offset: base.mining_bar.y_offset,
                z: base.mining_bar.z,
                color: parse_hex_color(&base.mining_bar.color),
            },
            toast: ToastConfig {
                lifetime: base.toast.lifetime,
                font_size: base.toast.font_size,
                color: parse_hex_color(&base.toast.color),
                bottom_px: base.toast.bottom_px,
            },
            tile_highlight: TileHighlightConfig {
                color: parse_hex_color(&base.tile_highlight.color),
                alpha: base.tile_highlight.alpha,
                z: base.tile_highlight.z,
            },
            deposit_sprite: DepositSpriteConfig {
                scale_ratio: base.deposit_sprite.scale_ratio,
                z: base.deposit_sprite.z,
                fallback_color: parse_hex_color(&base.deposit_sprite.fallback_color),
            },
            decoration: DecorationConfig {
                tree_z: base.decoration.tree_z,
                rock_z: base.decoration.rock_z,
                tree_color: parse_hex_color(&base.decoration.tree_color),
                rock_color: parse_hex_color(&base.decoration.rock_color),
            },
            decorations,
            chunk_colors: ChunkColorsConfig {
                even: parse_hex_color(&base.chunk_colors.even),
                odd: parse_hex_color(&base.chunk_colors.odd),
            },
            ghost: GhostConfig {
                tint_r: base.ghost.tint_r,
                tint_g: base.ghost.tint_g,
                tint_b: base.ghost.tint_b,
                tint_a: base.ghost.tint_a,
            },
            building: BuildingVisualConfig {
                owner_color: parse_hex_color(&base.building.owner_color),
                level_color: parse_hex_color(&base.building.level_color),
            },
        }
    }
}

#[derive(Clone, Deserialize)]
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
    #[serde(default)]
    decorations: Vec<DecorationTypeEntry>,
    chunk_colors: ChunkColorsEntry,
    ghost: GhostEntry,
    building: BuildingEntry,
}

#[derive(Clone, Deserialize)]
struct DecorationTypeEntry {
    kind: String,
    min_size: f32,
    max_size: f32,
    scatter: f32,
    density: f32,
    z: f32,
    color: String,
    shape: String,
}

#[derive(Clone, Deserialize)]
struct HpBarEntry {
    width: f32,
    height: f32,
    y_offset: f32,
    z: f32,
    color_high: String,
    color_mid: String,
    color_low: String,
}
#[derive(Clone, Deserialize)]
struct BeltItemEntry {
    width: f32,
    height: f32,
    z: f32,
}
#[derive(Clone, Deserialize)]
struct UnitEntry {
    width: f32,
    height: f32,
    z: f32,
    owner_color: String,
}
#[derive(Clone, Deserialize)]
struct PlayerEntry {
    width: f32,
    height: f32,
    z: f32,
}
#[derive(Clone, Deserialize)]
struct BuilderEntry {
    width: f32,
    height: f32,
    z: f32,
}
#[derive(Clone, Deserialize)]
struct EnemyEntry {
    boss_size: f32,
    tank_size: f32,
    default_size: f32,
    spawn_z: f32,
}
#[derive(Clone, Deserialize)]
struct ProjectileEntry {
    scale: f32,
}
#[derive(Clone, Deserialize)]
struct MiningBarEntry {
    width: f32,
    height: f32,
    y_offset: f32,
    z: f32,
    color: String,
}
#[derive(Clone, Deserialize)]
struct ToastEntry {
    lifetime: f32,
    font_size: f32,
    color: String,
    bottom_px: f32,
}
#[derive(Clone, Deserialize)]
struct TileHighlightEntry {
    color: String,
    alpha: f32,
    z: f32,
}
#[derive(Clone, Deserialize)]
struct DepositSpriteEntry {
    scale_ratio: f32,
    z: f32,
    fallback_color: String,
}
#[derive(Clone, Deserialize)]
struct DecorationEntry {
    tree_z: f32,
    rock_z: f32,
    tree_color: String,
    rock_color: String,
}
#[derive(Clone, Deserialize)]
struct ChunkColorsEntry {
    even: String,
    odd: String,
}
#[derive(Clone, Deserialize)]
struct GhostEntry {
    tint_r: f32,
    tint_g: f32,
    tint_b: f32,
    tint_a: f32,
}
#[derive(Clone, Deserialize)]
struct BuildingEntry {
    owner_color: String,
    level_color: String,
}




