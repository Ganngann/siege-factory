use bevy::prelude::*;

use crate::economy::components::Player;
use crate::economy::game_components::GroundItemStack;
use crate::economy::resource::{Inventory, ResourceId};
use crate::events::SpawnGroundItemEvent;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::TextureCache;

pub fn spawn_ground_item_visual(
    on: On<SpawnGroundItemEvent>,
    mut commands: Commands,
    cfg: Res<MapConfig>,
    textures: Res<TextureCache>,
) {
    let ev = on.event();
    let tile_size = cfg.tile_size;
    let wx = ev.position.x as f32 * tile_size;
    let wy = ev.position.y as f32 * tile_size;
    let tex = textures.base(&ev.resource_id);

    commands.spawn((
        GroundItemStack {
            resource_id: ev.resource_id.clone(),
            amount: ev.amount,
        },
        TilePosition {
            x: ev.position.x,
            y: ev.position.y,
        },
        Transform::from_xyz(wx, wy, 2.0),
        Visibility::default(),
        Sprite {
            image: tex,
            custom_size: Some(Vec2::new(tile_size * 0.6, tile_size * 0.6)),
            ..default()
        },
    ));
}

pub fn player_pickup_ground_items(
    mut commands: Commands,
    player_q: Query<&TilePosition, With<Player>>,
    items_q: Query<(Entity, &GroundItemStack, &TilePosition)>,
    mut player_inv_q: Query<&mut Inventory, With<Player>>,
) {
    let Ok(player_tile) = player_q.single() else {
        return;
    };
    let Ok(mut inv) = player_inv_q.single_mut() else {
        return;
    };

    for (entity, item, tile) in &items_q {
        let dx = (player_tile.x - tile.x).abs();
        let dy = (player_tile.y - tile.y).abs();
        if dx <= 1 && dy <= 1 {
            inv.add(&ResourceId(item.resource_id.clone()), item.amount);
            commands.entity(entity).despawn();
        }
    }
}
