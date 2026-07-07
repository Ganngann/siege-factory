use bevy::prelude::*;

use crate::core::game_state::IsFreshGame;
use crate::core::utils::tile_to_world;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{Building, OccupiedTiles};
use crate::economy::game_components::{Capsule, CurrentTier};
use crate::economy::resource::Inventory;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::TextureCache;

#[derive(Debug, Clone, Resource)]
pub struct CapsuleConfig {
    pub spawn_tile_x: i32,
    pub spawn_tile_y: i32,
    pub building_kind: String,
}

impl Default for CapsuleConfig {
    fn default() -> Self {
        Self {
            spawn_tile_x: 0,
            spawn_tile_y: 0,
            building_kind: "genesis_ark".to_string(),
        }
    }
}

impl CapsuleConfig {
    pub fn tier_texture_stems(&self, registry: &BuildingRegistry) -> Vec<String> {
        let Some(def) = registry.get(&self.building_kind) else {
            return Vec::new();
        };
        def.tiers.iter().map(|t| t.texture.clone()).collect()
    }
}

pub fn spawn_capsule(
    mut commands: Commands,
    cfg: Res<MapConfig>,
    textures: Res<TextureCache>,
    capsule_cfg: Res<CapsuleConfig>,
    fresh: Res<IsFreshGame>,
) {
    if !fresh.0 {
        return;
    }

    let (sx, sy) = (capsule_cfg.spawn_tile_x, capsule_cfg.spawn_tile_y);
    let tile_size = cfg.tile_size;
    let pos = tile_to_world(sx, sy, tile_size);

    let stem = &capsule_cfg.building_kind;
    let tex = textures.base(stem);

    commands.spawn((
        Capsule,
        CurrentTier(0),
        Building {
            kind: capsule_cfg.building_kind.clone(),
            name: capsule_cfg.building_kind.clone(),
        },
        OccupiedTiles(vec![(sx, sy)]),
        Inventory::new(),
        TilePosition { x: sx, y: sy },
        Transform::from_xyz(pos.x, pos.y, 5.0),
        Visibility::default(),
        Sprite {
            image: tex,
            custom_size: Some(Vec2::new(tile_size, tile_size)),
            ..default()
        },
    ));
}

pub fn update_capsule_visual(
    mut capsule_q: Query<
        (&CurrentTier, &mut Sprite),
        (With<Capsule>, Changed<CurrentTier>),
    >,
    textures: Res<TextureCache>,
    building_registry: Res<BuildingRegistry>,
    capsule_cfg: Res<CapsuleConfig>,
) {
    for (tier, mut sprite) in &mut capsule_q {
        let Some(def) = building_registry.get(&capsule_cfg.building_kind) else {
            continue;
        };
        if tier.0 >= def.tiers.len() {
            continue;
        }
        let stem = &def.tiers[tier.0].texture;
        if let Some(handle) = textures.base.get(stem) {
            sprite.image = handle.clone();
        }
    }
}
