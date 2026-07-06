use crate::combat::Projectile;
use crate::economy::belt::BeltSlots;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{
    BuildMode, Building, Direction, HasHpBar, HpBarChild, UnbuiltBuilding, Unit,
};
use crate::economy::unit_config::UnitConfig;
use crate::enemy::components::{Enemy, Health};
use crate::events::SpawnProjectileEvent;
use crate::map::components::HoveredTile;
use crate::map::config::MapConfig;
use crate::unit::{Soldier, Worker};
use crate::rendering::{ShapeCache, TextureCache};
use bevy::prelude::*;

#[derive(Component)]
pub struct TileHighlight;

pub fn tile_highlight(
    mut commands: Commands,
    build_mode: Res<BuildMode>,
    hovered: Res<HoveredTile>,
    cfg: Res<MapConfig>,
    existing: Query<Entity, With<TileHighlight>>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }

    if build_mode.0.is_some() {
        return;
    }

    let Some(pos) = hovered.0 else { return };

    commands.spawn((
        TileHighlight,
        Mesh2d(shapes.square.clone()),
        MeshMaterial2d(materials.add(Color::srgba(1.0, 1.0, 1.0, 0.15))),
        Transform::from_xyz(
            pos.x as f32 * cfg.tile_size,
            pos.y as f32 * cfg.tile_size,
            0.5,
        ),
    ));
}

pub fn ensure_hp_bars(
    mut commands: Commands,
    entities: Query<(Entity, &Health), (Without<HasHpBar>, Without<HpBarChild>)>,
) {
    for (entity, _health) in &entities {
        commands
            .entity(entity)
            .insert(HasHpBar)
            .with_children(|parent| {
                parent.spawn((
                    HpBarChild,
                    Sprite::from_color(Color::srgb(0.3, 1.0, 0.3), Vec2::new(24.0, 3.0)),
                    Transform::from_xyz(0.0, 20.0, 10.0),
                ));
            });
    }
}

pub fn update_hp_bars(
    health_q: Query<(&Health, &Children)>,
    mut sprite_q: Query<&mut Sprite, With<HpBarChild>>,
) {
    for (health, children) in health_q.iter() {
        for child in children.iter() {
            if let Ok(mut sprite) = sprite_q.get_mut(child) {
                let ratio = health.current as f32 / health.max as f32;
                let color = if ratio > 0.6 {
                    Color::srgb(0.3, 1.0, 0.3)
                } else if ratio > 0.3 {
                    Color::srgb(1.0, 0.8, 0.2)
                } else {
                    Color::srgb(1.0, 0.2, 0.2)
                };
                sprite.color = color;
                sprite.custom_size = Some(Vec2::new(24.0 * ratio, 3.0));
            }
        }
    }
}

pub fn sync_belt_slot_sprites(
    mut commands: Commands,
    textures: Res<TextureCache>,
    cfg: Res<MapConfig>,
    mut belt_query: Query<&mut BeltSlots>,
) {
    for mut bs in belt_query.iter_mut() {
        let mut to_create: Vec<(usize, crate::economy::resource::ResourceId)> = Vec::new();
        for (slot_idx, item) in bs.items.iter().enumerate() {
            if item.is_some() && bs.slot_sprites[slot_idx].is_none() {
                to_create.push((slot_idx, item.as_ref().unwrap().resource_id.clone()));
            }
        }
        let mut to_destroy: Vec<usize> = Vec::new();
        for (slot_idx, sprite) in bs.slot_sprites.iter().enumerate() {
            if sprite.is_some() && bs.items[slot_idx].is_none() {
                to_destroy.push(slot_idx);
            }
        }
        for (slot_idx, resource_id) in to_create {
            let tex = textures.base(&resource_id.0);
            let target = bs.slot_positions[slot_idx];
            let entry = if slot_idx == 0 {
                let (dx, dy) = bs.direction.offset();
                let dir_vec = Vec2::new(dx as f32, dy as f32);
                target - dir_vec * (cfg.tile_size / bs.items.len() as f32)
            } else {
                bs.slot_positions[slot_idx - 1]
            };
            let entity = commands
                .spawn((
                    Sprite {
                        image: tex,
                        custom_size: Some(Vec2::new(20.0, 20.0)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(entry.x, entry.y, 2.5)),
                ))
                .id();
            bs.slot_sprites[slot_idx] = Some(entity);
        }
        for slot_idx in to_destroy {
            if let Some(entity) = bs.slot_sprites[slot_idx].take() {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn animate_belt_positions(
    time: Res<Time>,
    cfg: Res<MapConfig>,
    belt_query: Query<&BeltSlots>,
    mut sprite_query: Query<&mut Transform>,
) {
    let dt = time.delta_secs();
    let tile_size = cfg.tile_size;

    for bs in belt_query.iter() {
        for (slot_idx, sprite_entity) in bs.slot_sprites.iter().enumerate() {
            if let Some(entity) = sprite_entity {
                if let Ok(mut transform) = sprite_query.get_mut(*entity) {
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

pub fn attach_unit_visuals(
    mut commands: Commands,
    units: Query<(Entity, &Unit, Option<&Worker>, Option<&Soldier>), Without<Sprite>>,
    textures: Res<TextureCache>,
    unit_cfg: Res<UnitConfig>,
) {
    for (entity, _unit, worker, _soldier) in units.iter() {
        let kind = if worker.is_some() {
            "worker"
        } else {
            "soldier"
        };
        let stem = unit_cfg
            .get(kind)
            .map(|d| d.texture_stem.as_str())
            .unwrap_or(kind);
        let img = textures.base(stem);
        let size = Vec2::new(48.0, 48.0);
        commands.entity(entity).insert((
            Sprite {
                image: img,
                custom_size: Some(size),
                ..default()
            },
            Visibility::default(),
        ));
        commands.entity(entity).with_children(|parent| {
            if let Some(tex) = textures.owner(stem) {
                parent.spawn((
                    Sprite {
                        image: tex,
                        custom_size: Some(size),
                        color: Color::srgb(0.2, 0.4, 0.8),
                        ..default()
                    },
                    Transform::default(),
                ));
            }
        });
    }
}

pub fn attach_building_visuals(
    mut commands: Commands,
    buildings: Query<
        (Entity, &Building),
        (Without<Sprite>, Without<BeltSlots>, Without<UnbuiltBuilding>),
    >,
    unbuilt: Query<
        (Entity, &Building),
        (With<UnbuiltBuilding>, Without<Sprite>, Without<BeltSlots>),
    >,
    belts: Query<(Entity, &Building, &BeltSlots), Without<Sprite>>,
    cfg: Res<MapConfig>,
    textures: Res<TextureCache>,
    registry: Res<BuildingRegistry>,
) {
    for (entity, building) in buildings.iter() {
        let Some(def) = registry.get(&building.kind) else {
            continue;
        };
        let tw = def.tile_size.0 as f32;
        let th = def.tile_size.1 as f32;
        let size = Vec2::new(tw * cfg.tile_size, th * cfg.tile_size);
        let stem = &def.texture_stem;
        commands.entity(entity).insert((Sprite {
            image: textures.base(stem),
            custom_size: Some(size),
            ..default()
        },));
        commands.entity(entity).with_children(|parent| {
            if let Some(tex) = textures.owner(stem) {
                parent.spawn((
                    Sprite {
                        image: tex,
                        custom_size: Some(size),
                        color: Color::srgb(0.2, 0.4, 0.8),
                        ..default()
                    },
                    Transform::default(),
                ));
            }
            if let Some(tex) = textures.level(stem) {
                parent.spawn((
                    Sprite {
                        image: tex,
                        custom_size: Some(size),
                        color: Color::srgb(0.2, 0.8, 0.2),
                        ..default()
                    },
                    Transform::default(),
                ));
            }
        });
    }

    for (entity, building) in unbuilt.iter() {
        let Some(def) = registry.get(&building.kind) else {
            continue;
        };
        let tw = def.tile_size.0 as f32;
        let th = def.tile_size.1 as f32;
        let size = Vec2::new(tw * cfg.tile_size, th * cfg.tile_size);
        let stem = &def.texture_stem;
        commands.entity(entity).insert((Sprite {
            image: textures.base(stem),
            custom_size: Some(size),
            color: Color::srgba(1.0, 0.3, 0.3, 0.5),
            ..default()
        },));
    }

    for (entity, building, _slots) in belts.iter() {
        let Some(def) = registry.get(&building.kind) else {
            continue;
        };
        let tw = def.tile_size.0 as f32;
        let th = def.tile_size.1 as f32;
        let size = Vec2::new(tw * cfg.tile_size, th * cfg.tile_size);
        let stem = &def.texture_stem;
        commands.entity(entity).insert((Sprite {
            image: textures.base(stem),
            custom_size: Some(size),
            ..default()
        },));
    }
}

pub fn attach_enemy_visuals(
    mut commands: Commands,
    enemies: Query<(Entity, &Enemy), Without<Sprite>>,
    textures: Res<TextureCache>,
) {
    for (entity, enemy) in enemies.iter() {
        let stem = &enemy.kind;
        let tex = textures.base(stem);
        let size = match stem as &str {
            "boss" => 48.0,
            "tank" => 36.0,
            _ => 28.0,
        };
        commands.entity(entity).insert((
            Sprite {
                image: tex,
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
        ));
    }
}

pub fn spawn_projectile_visual(
    on: On<SpawnProjectileEvent>,
    mut commands: Commands,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ev = on.event();
    commands.spawn((
        Projectile {
            target: ev.target,
            speed: ev.speed,
            damage: ev.damage,
        },
        Mesh2d(shapes.circle.clone()),
        MeshMaterial2d(materials.add(ev.color)),
        Transform::from_translation(ev.origin).with_scale(Vec3::splat(0.3)),
    ));
}

pub fn direction_arrow(dir: Direction) -> &'static str {
    match dir {
        Direction::East => ">",
        Direction::North => "^",
        Direction::West => "<",
        Direction::South => "v",
    }
}
