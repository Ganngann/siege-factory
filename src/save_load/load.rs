use bevy::prelude::*;
use std::path::PathBuf;

use crate::agriculture::components::Farm;
use crate::core::game_state::{GameState, IsFreshGame};
use crate::core::toast::ToastQueue;
use crate::core::utils::tile_to_world;
use crate::economy::belt::{BeltSlots, ItemOnBelt};
use crate::economy::building::{BuildingRegistry, attach_power_components};
use crate::economy::components::{
    Active, Assembler, Building, Direction, HpBarChild, OccupiedTiles, PeacefulMode, Player,
    Sorter, Splitter, Storage, TurretCombat, Unit,
};
use crate::economy::game_components::Miner;
use crate::economy::resource::{Inventory, ResourceId, ResourceRegistry};
use crate::enemy::components::{Enemy as EnemyComponent, Health, LastWave, WaveState};
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::map::systems::spawn_single_chunk_visuals;
use crate::map::tile_grid::{CHUNK_SIZE, ChunkGrid};
use crate::rendering::{ShapeCache, TextureCache};
use crate::unit::{Worker, WorkerState};

use super::{LoadBuffer, SaveData, load_data};

pub fn read_save_file(
    mut save_mgr: ResMut<super::SaveManager>,
    mut buf: ResMut<LoadBuffer>,
    mut toast: ResMut<ToastQueue>,
) {
    let path = match &save_mgr.load_requested {
        Some(p) => PathBuf::from(p),
        None => return,
    };
    *save_mgr = super::SaveManager {
        load_requested: None,
    };
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            toast.0.push(format!("Load failed: {e}"));
            return;
        }
    };
    let data: SaveData = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            toast.0.push(format!("Save file corrupt: {e}"));
            return;
        }
    };
    buf.data = Some(data);
}

pub fn load_chunks(
    buf: Res<LoadBuffer>,
    mut chunk_grid: ResMut<ChunkGrid>,
    cfg: Res<MapConfig>,
    res_registry: Res<ResourceRegistry>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    textures: Res<TextureCache>,
    mut commands: Commands,
) {
    let data = load_data!(buf);
    chunk_grid.clear();
    chunk_grid.set_seed(data.game_seed);
    for ((cx, cy), deposits) in &data.chunk_deposits {
        let chunk = chunk_grid.ensure_chunk_mut(*cx, *cy);
        chunk.deposits = deposits.clone();
    }

    let (hx, hy) = data
        .buildings
        .iter()
        .find(|b| b.kind == "hq")
        .map(|b| (b.tile_x, b.tile_y))
        .unwrap_or((0, 0));
    let chunk_size = CHUNK_SIZE as i32;
    let hq_cx = hx.div_euclid(chunk_size);
    let hq_cy = hy.div_euclid(chunk_size);

    for cx in (hq_cx - 10)..=(hq_cx + 10) {
        for cy in (hq_cy - 10)..=(hq_cy + 10) {
            spawn_single_chunk_visuals(
                &mut commands,
                &mut chunk_grid,
                &cfg,
                &res_registry,
                &shapes,
                &mut materials,
                &mut meshes,
                &textures,
                cx,
                cy,
            );
        }
    }
}

pub fn spawn_camera(commands: &mut Commands, transform: Transform) {
    commands.spawn((
        Camera2d,
        bevy::ui::IsDefaultUiCamera,
        transform,
        bevy_pancam::PanCam {
            grab_buttons: vec![MouseButton::Middle],
            speed: 500.0,
            min_scale: 0.3,
            max_scale: 3.0,
            ..default()
        },
    ));
}

pub fn spawn_fresh_camera(mut commands: Commands, cfg: Res<MapConfig>, buf: Res<LoadBuffer>) {
    if buf.data.is_some() {
        return;
    }
    let (hx, hy) = cfg.player_start_position;
    let pos = tile_to_world(hx, hy, cfg.tile_size);
    spawn_camera(&mut commands, Transform::from_xyz(pos.x, pos.y, 100.0));
}

pub fn load_camera(buf: Res<LoadBuffer>, mut commands: Commands) {
    let data = load_data!(buf);
    spawn_camera(
        &mut commands,
        Transform::from_xyz(data.camera.x, data.camera.y, 100.0)
            .with_scale(Vec3::splat(data.camera.scale)),
    );
}

pub fn load_buildings(
    buf: Res<LoadBuffer>,
    mut commands: Commands,
    cfg: Res<MapConfig>,
    registry: Res<BuildingRegistry>,
) {
    let data = load_data!(buf);
    let tile_size = cfg.tile_size;

    for bs in &data.buildings {
        let (tw, th) = if bs.kind == "hq" { (2, 2) } else { (1, 1) };
        let cx = (bs.tile_x as f32 + (tw as f32 - 1.0) * 0.5) * tile_size;
        let cy = (bs.tile_y as f32 + (th as f32 - 1.0) * 0.5) * tile_size;
        let inv = if let Some(ref items) = bs.inventory {
            let mut i = Inventory::with_capacity(bs.inventory_capacity);
            for (res, amount) in items {
                i.add(&ResourceId(res.clone()), *amount);
            }
            i
        } else if bs.inventory_capacity > 0 {
            Inventory::with_capacity(bs.inventory_capacity)
        } else {
            Inventory::new()
        };
        let tf = Transform::from_xyz(cx, cy, 2.0);
        let tile_pos = TilePosition {
            x: bs.tile_x,
            y: bs.tile_y,
        };
        let occupied = OccupiedTiles(bs.occupied.clone());
        let building = Building {
            kind: bs.kind.clone(),
            name: bs.kind.clone(),
        };

        if bs.kind == "hq" {
            commands.spawn((
                Player,
                building,
                inv,
                tf,
                tile_pos,
                Health {
                    current: cfg.player_hp,
                    max: cfg.player_hp,
                },
            ));
        } else if bs.kind == "miner" {
            if let Some(a) = &bs.assembler {
                let mut e = commands.spawn((
                    Miner,
                    Assembler {
                        production_timer: a.production_timer,
                        interval: a.interval,
                        recipe_id: a.recipe_id.clone(),
                    },
                    building,
                    inv,
                    occupied,
                    tf,
                    tile_pos,
                    Active(true),
                ));
                if let Some(def) = registry.get(&bs.kind) {
                    attach_power_components(&mut e, def);
                }
            } else {
                bevy::log::warn!("Skipped miner at ({}, {}): missing assembler data", bs.tile_x, bs.tile_y);
            }
        } else if let Some(a) = &bs.assembler {
            let mut e = commands.spawn((
                Assembler {
                    production_timer: a.production_timer,
                    interval: a.interval,
                    recipe_id: a.recipe_id.clone(),
                },
                building,
                inv,
                occupied,
                tf,
                tile_pos,
                Active(true),
            ));
            if let Some(def) = registry.get(&bs.kind) {
                attach_power_components(&mut e, def);
            }
        } else if let Some(f) = &bs.farm {
            commands.spawn((
                Farm {
                    crop_index: 0,
                    crop_types: f.crop_types.clone(),
                },
                building,
                inv,
                occupied,
                tf,
                tile_pos,
                Active(true),
            ));
        } else if bs.belt.is_some() || bs.splitter.is_some() || bs.sorter.is_some() {
            if let Some(b) = &bs.belt {
                let slot_positions = crate::economy::belt::compute_slot_positions(
                    bs.tile_x,
                    bs.tile_y,
                    b.direction,
                    b.slots.len() as u32,
                    tile_size,
                );
                let angle = match b.direction {
                    Direction::East => 0.0,
                    Direction::North => std::f32::consts::FRAC_PI_2,
                    Direction::West => std::f32::consts::PI,
                    Direction::South => -std::f32::consts::FRAC_PI_2,
                };
                let belt_tf =
                    Transform::from_xyz(cx, cy, 2.0).with_rotation(Quat::from_rotation_z(angle));
                let mut items: Vec<Option<ItemOnBelt>> = Vec::new();
                for item_save in &b.slots {
                    if let Some(item) = item_save {
                        items.push(Some(ItemOnBelt {
                            resource_id: ResourceId(item.resource.clone()),
                            acc: item.acc,
                        }));
                    } else {
                        items.push(None);
                    }
                }
                let slot_sprites: Vec<Option<Entity>> = vec![None; items.len()];
                let belt_comp = BeltSlots {
                    direction: b.direction,
                    items,
                    slot_sprites,
                    slot_positions,
                    speed: b.speed,
                };
                if let Some(sp) = &bs.splitter {
                    commands.spawn((
                        belt_comp,
                        building,
                        inv,
                        occupied,
                        belt_tf,
                        tile_pos,
                        Splitter {
                            counter: sp.counter,
                            outputs: sp.outputs,
                            input_direction: sp.input_direction,
                        },
                        Active(true),
                    ));
                } else if let Some(so) = &bs.sorter {
                    commands.spawn((
                        belt_comp,
                        building,
                        inv,
                        occupied,
                        belt_tf,
                        tile_pos,
                        Sorter {
                            filter: ResourceId(so.filter.clone()),
                            inverted: so.inverted,
                        },
                        Active(true),
                    ));
                } else {
                    commands.spawn((
                        belt_comp,
                        building,
                        inv,
                        occupied,
                        belt_tf,
                        tile_pos,
                        Active(true),
                    ));
                }
            } else {
                bevy::log::warn!("Skipped belt at ({}, {}): missing belt data", bs.tile_x, bs.tile_y);
            }
        } else if bs.kind == "turret" {
            if let Some(t) = &bs.turret {
                commands.spawn((
                    TurretCombat {
                        damage: t.damage,
                        range_sq: t.range_sq,
                        fire_interval: t.fire_interval,
                        timer: t.timer,
                        projectile_speed: t.projectile_speed,
                    },
                    building,
                    inv,
                    occupied,
                    tf,
                    tile_pos,
                    Active(true),
                ));
            } else {
                bevy::log::warn!("Skipped turret at ({}, {}): missing turret data", bs.tile_x, bs.tile_y);
            }
        } else if bs.storage {
            commands.spawn((Storage, building, inv, occupied, tf, tile_pos, Active(true)));
        } else {
            commands.spawn((building, inv, occupied, tf, tile_pos, Active(true)));
        }
    }
}

pub fn load_enemies(buf: Res<LoadBuffer>, mut commands: Commands, cfg: Res<MapConfig>) {
    let data = load_data!(buf);
    for es in &data.enemies {
        let entity = commands
            .spawn((
                EnemyComponent {
                    kind: es.kind.clone(),
                },
                Health {
                    current: es.hp,
                    max: es.max_hp,
                },
                Transform::from_xyz(es.x, es.y, 3.0),
                TilePosition {
                    x: (es.x / cfg.tile_size) as i32,
                    y: (es.y / cfg.tile_size) as i32,
                },
            ))
            .id();
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                HpBarChild,
                Sprite {
                    custom_size: Some(Vec2::new(24.0, 3.0)),
                    color: Color::srgb(0.2, 1.0, 0.2),
                    ..default()
                },
                Transform::from_xyz(0.0, 20.0, 1.0),
            ));
        });
    }
}

pub fn load_units(buf: Res<LoadBuffer>, mut commands: Commands, cfg: Res<MapConfig>) {
    let data = load_data!(buf);
    for us in &data.units {
        if us.kind == "worker" {
            commands.spawn((
                Worker {
                    state: WorkerState::Idle,
                    mining_timer: us.worker_timer.unwrap_or(0.0),
                },
                Unit,
                Health {
                    current: us.hp,
                    max: us.max_hp,
                },
                Transform::from_xyz(us.x, us.y, 2.5),
                TilePosition {
                    x: (us.x / cfg.tile_size) as i32,
                    y: (us.y / cfg.tile_size) as i32,
                },
            ));
        } else {
            commands.spawn((
                crate::unit::Soldier {
                    attack_cooldown: us.soldier_cooldown.unwrap_or(0.0),
                },
                Unit,
                Health {
                    current: us.hp,
                    max: us.max_hp,
                },
                Transform::from_xyz(us.x, us.y, 2.5),
                TilePosition {
                    x: (us.x / cfg.tile_size) as i32,
                    y: (us.y / cfg.tile_size) as i32,
                },
            ));
        }
    }
}

pub fn load_finalize(
    mut buf: ResMut<LoadBuffer>,
    mut wave: ResMut<WaveState>,
    mut last_wave: ResMut<LastWave>,
    mut peaceful: ResMut<PeacefulMode>,
    mut fresh_game: ResMut<IsFreshGame>,
    mut next_state: ResMut<NextState<GameState>>,
    mut toast: ResMut<ToastQueue>,
) {
    let data = match &buf.data {
        Some(d) => d,
        None => {
            buf.data = None;
            next_state.set(GameState::Menu);
            return;
        }
    };
    wave.timer = data.wave.timer;
    wave.wave = data.wave.wave;
    wave.spawn_timer = data.wave.spawn_timer;
    last_wave.0 = data.wave.last_wave;
    peaceful.0 = true;
    fresh_game.0 = false;
    buf.data = None;
    next_state.set(GameState::Playing);
    toast.0.push("Game loaded".to_string());
}
