use std::collections::HashMap;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use crate::economy::resource::ResourceId;
use crate::economy::systems::Direction;
use crate::events::SpawnBeltItemEvent;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::{material_from_color, ShapeCache};

#[derive(Component)]
pub struct BeltSlots {
    pub direction: Direction,
    pub slots: Vec<Option<Entity>>,
    pub slot_positions: Vec<Vec2>,
    pub speed: f32,
}

#[derive(Component)]
pub struct BeltItem {
    pub resource: ResourceId,
    pub acc: f32,
}

pub fn compute_slot_positions(
    tx: u32,
    ty: u32,
    direction: Direction,
    num_slots: u32,
    tile_size: f32,
) -> Vec<Vec2> {
    let center = Vec2::new(tx as f32 * tile_size, ty as f32 * tile_size);
    let (dx, dy) = direction.offset();
    let dir_vec = Vec2::new(dx as f32, dy as f32);
    (0..num_slots)
        .map(|i| {
            let fraction = (i as f32 + 0.5) / num_slots as f32;
            let offset = (fraction - 0.5) * tile_size;
            center + dir_vec * offset
        })
        .collect()
}

pub fn belt_item_placer(
    mut events: EventReader<SpawnBeltItemEvent>,
    mut belt_query: Query<(Entity, &TilePosition, &mut BeltSlots)>,
    mut commands: Commands,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cfg: Res<MapConfig>,
) {
    let tile_size = cfg.tile_size;
    let belt_map: HashMap<(u32, u32), Entity> =
        belt_query.iter().map(|(e, pos, _)| ((pos.x, pos.y), e)).collect();

    for ev in events.read() {
        let mut placed = false;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let ax = ev.source_tile.x.wrapping_add_signed(dx);
            let ay = ev.source_tile.y.wrapping_add_signed(dy);
            if let Some(&belt_entity) = belt_map.get(&(ax, ay)) {
                if let Ok((_, _, mut bs)) = belt_query.get_mut(belt_entity) {
                    if let Some(free_idx) = bs.slots.iter().position(|s| s.is_none()) {
                        let spawn_pos = Vec3::new(
                            ev.source_tile.x as f32 * tile_size,
                            ev.source_tile.y as f32 * tile_size,
                            2.5,
                        );
                        let color = match ev.resource {
                            ResourceId::Ore => Color::srgb(0.7, 0.5, 0.1),
                            ResourceId::Ammo => Color::srgb(0.8, 0.2, 0.2),
                            ResourceId::Energy => Color::srgb(0.2, 0.6, 0.8),
                        };
                        let item_entity = commands.spawn((
                            BeltItem { resource: ev.resource, acc: 0.0 },
                            ColorMesh2dBundle {
                                mesh: Mesh2dHandle(shapes.circle.clone()),
                                material: material_from_color(&mut materials, color),
                                transform: Transform::from_translation(spawn_pos)
                                    .with_scale(Vec3::splat(0.25)),
                                ..default()
                            },
                        )).id();
                        bs.slots[free_idx] = Some(item_entity);
                        placed = true;
                        break;
                    }
                }
            }
        }
        if !placed {
            // No free belt slot — item is backed up (no visual spawned)
        }
    }
}

pub fn advance_belt_slots(
    time: Res<Time>,
    mut belt_query: Query<(Entity, &TilePosition, &mut BeltSlots)>,
    mut item_query: Query<&mut BeltItem>,
) {
    let dt = time.delta_seconds();
    let belt_map: HashMap<(u32, u32), Entity> =
        belt_query.iter().map(|(e, pos, _)| ((pos.x, pos.y), e)).collect();

    let belt_data: Vec<(Entity, TilePosition, Direction, f32, usize)> = belt_query
        .iter()
        .map(|(e, pos, bs)| (e, *pos, bs.direction, bs.speed, bs.slots.len()))
        .collect();

    // Accumulate time on all items (capped at 1 slot duration)
    for (_, _, bs) in belt_query.iter() {
        let slot_duration = 1.0 / (bs.speed * bs.slots.len() as f32);
        for slot in &bs.slots {
            if let Some(item_entity) = slot {
                if let Ok(mut item) = item_query.get_mut(*item_entity) {
                    item.acc = (item.acc + dt).min(slot_duration);
                }
            }
        }
    }

    // Internal advancement: last → first within each belt
    for (belt_entity, _, _, speed, n_slots) in &belt_data {
        let slot_duration = 1.0 / (speed * *n_slots as f32);
        if let Ok((_, _, mut bs)) = belt_query.get_mut(*belt_entity) {
            for i in (0..bs.slots.len() - 1).rev() {
                if let Some(item_entity) = bs.slots[i] {
                    if bs.slots[i + 1].is_none() {
                        if let Ok(mut item) = item_query.get_mut(item_entity) {
                            if item.acc >= slot_duration {
                                bs.slots[i + 1] = bs.slots[i].take();
                                item.acc -= slot_duration;
                            }
                        }
                    }
                }
            }
        }
    }

    // Cross-belt transfers: last slot → next belt's first slot
    for (belt_entity, belt_pos, dir, speed, n_slots) in &belt_data {
        let slot_duration = 1.0 / (speed * *n_slots as f32);
        let (dx, dy) = dir.offset();
        let nx = belt_pos.x.wrapping_add_signed(dx);
        let ny = belt_pos.y.wrapping_add_signed(dy);
        if let Some(&next_belt) = belt_map.get(&(nx, ny)) {
            if *belt_entity == next_belt {
                continue;
            }
            if let Ok([(_, _, mut bs), (_, _, mut next_bs)]) =
                belt_query.get_many_mut([*belt_entity, next_belt])
            {
                let last = n_slots - 1;
                if let Some(item_entity) = bs.slots[last] {
                    if next_bs.slots[0].is_none() {
                        if let Ok(mut item) = item_query.get_mut(item_entity) {
                            if item.acc >= slot_duration {
                                next_bs.slots[0] = bs.slots[last].take();
                                item.acc -= slot_duration;
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn animate_belt_positions(
    time: Res<Time>,
    cfg: Res<MapConfig>,
    belt_query: Query<&BeltSlots>,
    mut item_query: Query<&mut Transform, With<BeltItem>>,
) {
    let dt = time.delta_seconds();
    let tile_size = cfg.tile_size;

    for bs in belt_query.iter() {
        for (slot_idx, occupant) in bs.slots.iter().enumerate() {
            if let Some(item_entity) = occupant {
                if let Ok(mut transform) = item_query.get_mut(*item_entity) {
                    let target = bs.slot_positions[slot_idx];
                    let current = Vec2::new(transform.translation.x, transform.translation.y);
                    let diff = target - current;
                    let step = bs.speed * tile_size * dt;
                    if diff.length() <= step {
                        transform.translation = Vec3::new(target.x, target.y, 2.5);
                    } else {
                        let new_pos = current + diff.normalize() * step;
                        transform.translation = Vec3::new(new_pos.x, new_pos.y, 2.5);
                    }
                }
            }
        }
    }
}
