use crate::combat::Projectile;
use crate::core::utils::tile_to_world;
use crate::economy::belt::BeltSlots;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{
    BuildMode, Building, Direction, HasHpBar, HpBarChild, Player, UnbuiltBuilding, Unit,
};
use crate::economy::game_components::{
    BeltVariant, CurrentTier, HasMiningProgress, MiningProgressChild,
};
use crate::economy::player::MiningTimer;
use crate::economy::unit_config::UnitConfig;
use crate::enemy::components::{Enemy, Health};
use crate::events::SpawnProjectileEvent;
use crate::map::components::HoveredTile;
use crate::map::config::MapConfig;
use crate::rendering::config::VisualsConfig;
use crate::rendering::{ShapeCache, TextureCache};
use crate::unit::{Soldier, Worker, WorkerState};
use bevy::prelude::*;

#[derive(Component)]
pub struct TileHighlight;

#[derive(Resource, Default)]
pub struct TileHighlightEntity(pub Option<Entity>);

pub fn tile_highlight(
    mut commands: Commands,
    build_mode: Res<BuildMode>,
    hovered: Res<HoveredTile>,
    cfg: Res<MapConfig>,
    config: Res<VisualsConfig>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut highlight: ResMut<TileHighlightEntity>,
) {
    if build_mode.0.is_some() {
        if let Some(entity) = highlight.0.take() {
            commands.entity(entity).despawn();
        }
        return;
    }

    let Some(pos) = hovered.0 else {
        if let Some(entity) = highlight.0.take() {
            commands.entity(entity).despawn();
        }
        return;
    };

    let world_pos = tile_to_world(pos.x, pos.y, cfg.tile_size);
    let world_x = world_pos.x;
    let world_y = world_pos.y;
    let z = config.tile_highlight.z;

    if let Some(entity) = highlight.0 {
        commands
            .entity(entity)
            .insert(Transform::from_xyz(world_x, world_y, z));
    } else {
        let entity = commands
            .spawn((
                TileHighlight,
                Mesh2d(shapes.square.clone()),
                MeshMaterial2d(materials.add(Color::srgba(
                    config.tile_highlight.color.to_srgba().red,
                    config.tile_highlight.color.to_srgba().green,
                    config.tile_highlight.color.to_srgba().blue,
                    config.tile_highlight.alpha,
                ))),
                Transform::from_xyz(world_x, world_y, z),
            ))
            .id();
        highlight.0 = Some(entity);
    }
}

pub fn ensure_hp_bars(
    mut commands: Commands,
    entities: Query<(Entity, &Health), (Without<HasHpBar>, Without<HpBarChild>)>,
    config: Res<VisualsConfig>,
) {
    for (entity, _health) in &entities {
        commands
            .entity(entity)
            .insert(HasHpBar)
            .with_children(|parent| {
                parent.spawn((
                    HpBarChild,
                    Sprite::from_color(
                        config.hp_bar.color_high,
                        Vec2::new(config.hp_bar.width, config.hp_bar.height),
                    ),
                    Transform::from_xyz(0.0, config.hp_bar.y_offset, config.hp_bar.z),
                ));
            });
    }
}

pub fn ensure_mining_progress_bars(
    mut commands: Commands,
    workers: Query<Entity, (With<Worker>, Without<HasMiningProgress>)>,
    players: Query<Entity, (With<Player>, Without<HasMiningProgress>)>,
    config: Res<VisualsConfig>,
) {
    for entity in workers.iter().chain(players.iter()) {
        commands
            .entity(entity)
            .insert(HasMiningProgress)
            .with_children(|parent| {
                parent.spawn((
                    MiningProgressChild,
                    Sprite::from_color(
                        config.mining_bar.color,
                        Vec2::new(config.mining_bar.width, config.mining_bar.height),
                    ),
                    Transform::from_xyz(0.0, config.mining_bar.y_offset, config.mining_bar.z),
                ));
            });
    }
}

pub fn update_mining_progress_bars(
    workers: Query<(&Worker, &Children)>,
    players: Query<&Children, (With<Player>, Without<Worker>)>,
    mut sprite_q: Query<&mut Sprite, With<MiningProgressChild>>,
    config: Res<VisualsConfig>,
    unit_cfg: Res<UnitConfig>,
    mining_timer: Res<MiningTimer>,
    map_cfg: Res<MapConfig>,
) {
    let interval = unit_cfg
        .get("worker")
        .map(|d| d.mine_interval_sec)
        .unwrap_or(3.0);
    for (worker, children) in workers.iter() {
        let ratio = if matches!(worker.state, WorkerState::Mining(_)) {
            (worker.mining_timer / interval).min(1.0)
        } else {
            0.0
        };
        for child in children.iter() {
            if let Ok(mut sprite) = sprite_q.get_mut(child) {
                sprite.custom_size = Some(Vec2::new(
                    config.mining_bar.width * ratio,
                    config.mining_bar.height,
                ));
            }
        }
    }

    let player_ratio = (mining_timer.0 / map_cfg.player_mining_interval).min(1.0);
    for children in players.iter() {
        for child in children.iter() {
            if let Ok(mut sprite) = sprite_q.get_mut(child) {
                sprite.custom_size = Some(Vec2::new(
                    config.mining_bar.width * player_ratio,
                    config.mining_bar.height,
                ));
            }
        }
    }
}

pub fn update_hp_bars(
    health_q: Query<(&Health, &Children)>,
    mut sprite_q: Query<&mut Sprite, With<HpBarChild>>,
    config: Res<VisualsConfig>,
) {
    for (health, children) in health_q.iter() {
        for child in children.iter() {
            if let Ok(mut sprite) = sprite_q.get_mut(child) {
                let ratio = health.current as f32 / health.max as f32;
                let color = if ratio > 0.6 {
                    config.hp_bar.color_high
                } else if ratio > 0.3 {
                    config.hp_bar.color_mid
                } else {
                    config.hp_bar.color_low
                };
                sprite.color = color;
                sprite.custom_size =
                    Some(Vec2::new(config.hp_bar.width * ratio, config.hp_bar.height));
            }
        }
    }
}

pub fn sync_belt_slot_sprites(
    mut commands: Commands,
    textures: Res<TextureCache>,
    cfg: Res<MapConfig>,
    config: Res<VisualsConfig>,
    mut belt_query: Query<&mut BeltSlots>,
) {
    for mut bs in belt_query.iter_mut() {
        let mut to_create: Vec<(usize, crate::economy::resource::ResourceId)> = Vec::new();
        for (slot_idx, item) in bs.items.iter().enumerate() {
            if let Some(item) = item {
                if bs.slot_sprites[slot_idx].is_none() {
                    to_create.push((slot_idx, item.resource_id.clone()));
                }
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
                        custom_size: Some(Vec2::new(
                            config.belt_item.width,
                            config.belt_item.height,
                        )),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(entry.x, entry.y, config.belt_item.z)),
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
    config: Res<VisualsConfig>,
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
                        transform.translation = Vec3::new(target.x, target.y, config.belt_item.z);
                    } else {
                        let new_pos = current + diff.normalize() * step;
                        transform.translation = Vec3::new(new_pos.x, new_pos.y, config.belt_item.z);
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
    config: Res<VisualsConfig>,
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
        let size = Vec2::new(config.unit.width, config.unit.height);
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
                        color: config.unit.owner_color,
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
        (
            Without<Sprite>,
            Without<BeltSlots>,
            Without<UnbuiltBuilding>,
        ),
    >,
    unbuilt: Query<
        (Entity, &Building),
        (With<UnbuiltBuilding>, Without<Sprite>, Without<BeltSlots>),
    >,
    belts: Query<(Entity, &Building, &BeltSlots, Option<&BeltVariant>), Without<Sprite>>,
    cfg: Res<MapConfig>,
    textures: Res<TextureCache>,
    registry: Res<BuildingRegistry>,
    config: Res<VisualsConfig>,
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
                        color: config.building.owner_color,
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
                        color: config.building.level_color,
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
            color: Color::srgba(
                config.ghost.tint_r,
                config.ghost.tint_g,
                config.ghost.tint_b,
                config.ghost.tint_a,
            ),
            ..default()
        },));
    }

    for (entity, building, _slots, belt_variant) in belts.iter() {
        let Some(def) = registry.get(&building.kind) else {
            continue;
        };
        let tw = def.tile_size.0 as f32;
        let th = def.tile_size.1 as f32;
        let size = Vec2::new(tw * cfg.tile_size, th * cfg.tile_size);
        let stem = &def.texture_stem;
        let tint = match belt_variant.unwrap_or(&BeltVariant::Normal) {
            BeltVariant::Normal => Color::WHITE,
            BeltVariant::Underground => Color::srgb(0.6, 0.6, 0.8),
            BeltVariant::Aerial => Color::srgb(0.8, 0.9, 1.0),
            BeltVariant::Curved => Color::srgb(0.9, 0.75, 0.6),
        };
        commands.entity(entity).insert((Sprite {
            image: textures.base(stem),
            custom_size: Some(size),
            color: tint,
            ..default()
        },));
    }
}

pub fn attach_enemy_visuals(
    mut commands: Commands,
    enemies: Query<(Entity, &Enemy), Without<Sprite>>,
    textures: Res<TextureCache>,
    config: Res<VisualsConfig>,
) {
    for (entity, enemy) in enemies.iter() {
        let stem = &enemy.kind;
        let tex = textures.base(stem);
        let size = if stem == "boss" {
            config.enemy.boss_size
        } else if stem == "tank" {
            config.enemy.tank_size
        } else {
            config.enemy.default_size
        };
        commands.entity(entity).insert((Sprite {
            image: tex,
            custom_size: Some(Vec2::new(size, size)),
            ..default()
        },));
    }
}

pub fn spawn_projectile_visual(
    on: On<SpawnProjectileEvent>,
    mut commands: Commands,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<VisualsConfig>,
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
        Transform::from_translation(ev.origin).with_scale(Vec3::splat(config.projectile.scale)),
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

pub fn update_tier_visuals(
    tier_q: Query<(Entity, &Building, &CurrentTier), Changed<CurrentTier>>,
    textures: Res<TextureCache>,
    registry: Res<BuildingRegistry>,
    cfg: Res<MapConfig>,
    mut commands: Commands,
) {
    for (entity, building, tier) in &tier_q {
        let Some(def) = registry.get(&building.kind) else {
            continue;
        };
        if tier.0 == 0 || tier.0 > def.tiers.len() {
            continue;
        }
        let tier_def = &def.tiers[tier.0 - 1];
        if tier_def.texture.is_empty() {
            continue;
        }
        let tw = def.tile_size.0 as f32;
        let th = def.tile_size.1 as f32;
        let size = Vec2::new(tw * cfg.tile_size, th * cfg.tile_size);
        commands.entity(entity).insert(Sprite {
            image: textures.base(&tier_def.texture),
            custom_size: Some(size),
            ..default()
        });
    }
}
