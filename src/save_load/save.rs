use bevy::prelude::*;
use std::collections::HashMap;

use crate::agriculture::components::Farm;
use crate::core::toast::ToastQueue;
use crate::economy::belt::BeltSlots;
use crate::economy::components::{
    Assembler, Building, OccupiedTiles, PowerConsumer, PowerPole, PowerProducer, Sorter, Splitter,
    Storage, TurretCombat, UnbuiltBuilding, Unit,
};
use crate::economy::resource::Inventory;
use crate::enemy::components::{Enemy as EnemyComponent, Health, LastWave, WaveState};
use crate::map::components::TilePosition;
use crate::map::tile_grid::ChunkGrid;
use crate::rendering::minimap::MinimapCamera;
use crate::unit::{Soldier, Worker, WorkerState};

use super::{
    BeltItemSave, BeltSave, BuildingSave, CameraSave, EnemySave, SaveData, SaveRequested, UnitSave,
    WorkerStateSave, save_path,
};

pub fn save_game(
    keys: Res<ButtonInput<KeyCode>>,
    mut save_req: ResMut<SaveRequested>,
    mut toast: ResMut<ToastQueue>,
    chunk_grid: Res<ChunkGrid>,
    wave: Res<WaveState>,
    last_wave: Res<LastWave>,
    camera: Query<&Transform, (With<Camera2d>, Without<MinimapCamera>)>,
    tile_positions: Query<&TilePosition>,
    buildings: Query<
        (
            &Building,
            &TilePosition,
            &OccupiedTiles,
            Option<&Health>,
            Option<&Inventory>,
            Option<&Assembler>,
            Option<&TurretCombat>,
            Option<&BeltSlots>,
            Option<&Storage>,
            Option<&Splitter>,
            Option<&Sorter>,
            Option<&Farm>,
            Option<&PowerConsumer>,
            Option<&PowerProducer>,
            Option<&PowerPole>,
        ),
        Without<UnbuiltBuilding>,
    >,
    enemies: Query<(&EnemyComponent, &Transform, &Health, &TilePosition)>,
    units: Query<
        (
            &Transform,
            &Health,
            &TilePosition,
            Option<&Soldier>,
            Option<&Worker>,
        ),
        With<Unit>,
    >,
) {
    if !keys.just_pressed(KeyCode::F5) && !save_req.0 {
        return;
    }
    save_req.0 = false;

    let mut data = SaveData {
        version: 1,
        game_seed: chunk_grid.seed(),
        camera: CameraSave {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        },
        wave: super::WaveSave {
            timer: wave.timer,
            wave: wave.wave,
            spawn_timer: wave.spawn_timer,
            last_wave: last_wave.0,
        },
        chunk_deposits: HashMap::new(),
        visited: HashMap::new(),
        buildings: Vec::new(),
        enemies: Vec::new(),
        units: Vec::new(),
    };

    if let Ok(tf) = camera.single() {
        data.camera.x = tf.translation.x;
        data.camera.y = tf.translation.y;
        data.camera.scale = tf.scale.x;
    }

    for ((cx, cy), chunk) in chunk_grid.generated_chunks_with_data() {
        let mut ref_grid = ChunkGrid::new(
            chunk_grid.seed(),
            chunk_grid.deposit_min_amount,
            chunk_grid.deposit_max_amount,
            chunk_grid.deposit_spawn_chance_pct,
            chunk_grid.deposit_min_per_chunk,
            chunk_grid.deposit_max_per_chunk,
            chunk_grid.deposit_distribution.clone(),
        );
        let original = ref_grid.ensure_chunk(*cx, *cy).deposits.clone();
        if chunk.deposits != original {
            data.chunk_deposits
                .insert((*cx, *cy), chunk.deposits.clone());
        }
        if !chunk.visited.is_empty() {
            data.visited.insert(
                (*cx, *cy),
                chunk.visited.iter().copied().collect(),
            );
        }
    }

    for (
        building,
        pos,
        occupied,
        hp,
        inventory,
        assembler,
        turret,
        belt,
        storage,
        splitter,
        sorter,
        farm,
        power_consumer,
        power_producer,
        power_pole,
    ) in buildings.iter()
    {
        let belt_save = belt.map(|b| {
            let slots: Vec<Option<BeltItemSave>> = b
                .items
                .iter()
                .map(|item| {
                    item.as_ref().map(|i| BeltItemSave {
                        resource: i.resource_id.0.clone(),
                        acc: i.acc,
                    })
                })
                .collect();
            BeltSave {
                direction: b.direction,
                speed: b.speed,
                slots,
            }
        });
        data.buildings.push(BuildingSave {
            kind: building.kind.clone(),
            tile_x: pos.x,
            tile_y: pos.y,
            occupied: occupied.0.clone(),
            hp: hp.map(|h| (h.current, h.max)),
            inventory: inventory.map(|inv| {
                inv.resources
                    .iter()
                    .map(|(r, a)| (r.0.clone(), *a))
                    .collect()
            }),
            inventory_capacity: inventory.map(|inv| inv.capacity).unwrap_or(0),
            assembler: assembler.map(|a| super::AssemblerSave {
                production_timer: a.production_timer,
                interval: a.interval,
                recipe_id: a.recipe_id.clone(),
            }),
            turret: turret.map(|t| super::TurretSave {
                damage: t.damage,
                range_sq: t.range_sq,
                fire_interval: t.fire_interval,
                timer: t.timer,
                projectile_speed: t.projectile_speed,
            }),
            belt: belt_save,
            storage: storage.is_some(),
            splitter: splitter.map(|s| super::SplitterSave {
                counter: s.counter,
                outputs: s.outputs,
                input_direction: s.input_direction,
            }),
            sorter: sorter.map(|s| super::SorterSave {
                filter: s.filter.0.clone(),
                inverted: s.inverted,
            }),
            farm: farm.map(|f| super::FarmSave {
                crop_types: f.crop_types.clone(),
            }),
            power_draw: power_consumer.map(|pc| pc.draw),
            power_generation: power_producer.map_or(0.0, |pp| pp.output),
            power_pole_range: power_pole.map_or(0.0, |pp| pp.range),
        });
    }

    for (enemy, tf, hp, _) in enemies.iter() {
        data.enemies.push(EnemySave {
            kind: enemy.kind.clone(),
            x: tf.translation.x,
            y: tf.translation.y,
            hp: hp.current,
            max_hp: hp.max,
        });
    }

    for (tf, hp, _pos, soldier, worker) in units.iter() {
        let kind = if soldier.is_some() {
            "soldier"
        } else {
            "worker"
        };
        let worker_state = worker.map(|w| match &w.state {
            WorkerState::Idle => WorkerStateSave::Idle,
            WorkerState::MovingToDeposit(e) => tile_positions
                .get(*e)
                .map(|pos| WorkerStateSave::MovingToDeposit {
                    target_tx: pos.x,
                    target_ty: pos.y,
                })
                .unwrap_or(WorkerStateSave::Idle),
            WorkerState::Mining(e) => tile_positions
                .get(*e)
                .map(|pos| WorkerStateSave::Mining {
                    target_tx: pos.x,
                    target_ty: pos.y,
                })
                .unwrap_or(WorkerStateSave::Idle),
        });
        data.units.push(UnitSave {
            kind: kind.to_string(),
            x: tf.translation.x,
            y: tf.translation.y,
            hp: hp.current,
            max_hp: hp.max,
            soldier_cooldown: soldier.map(|s| s.attack_cooldown),
            worker_timer: worker.map(|w| w.mining_timer),
            worker_state,
        });
    }

    let path = save_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match ron::to_string(&data) {
        Ok(content) => match std::fs::write(&path, content) {
            Ok(_) => toast.0.push("Game saved".to_string()),
            Err(e) => toast.0.push(format!("Save failed: {e}")),
        },
        Err(e) => toast.0.push(format!("Save serialization failed: {e}")),
    }
}
