use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::combat::Projectile;
use crate::core::game_state::{GameState, IsFreshGame};
use crate::core::toast::ToastQueue;
use crate::core::utils::{config_dir, silent_despawn};
use crate::economy::belt::{BeltSlots, ItemOnBelt};
use crate::economy::components::{
    Assembler, Building, Direction, Ghost, HQ, Miner, OccupiedTiles,
    ResourceDeposit, Splitter, Storage, Sorter, TurretCombat,
    Unit, HpBarChild, PeacefulMode, PanelModal, Active,
};
use crate::economy::resource::{ResourceId, Inventory, ResourceRegistry};
use crate::economy::ui::ResourceCountText;
use crate::enemy::components::{Enemy as EnemyComponent, Health, LastWave, WaveState};
use crate::map::components::{ChunkMember, TilePosition};
use crate::map::config::MapConfig;
use crate::map::systems::{spawn_single_chunk_visuals, ChunkMarker};
use crate::map::tile_grid::{ChunkGrid, CHUNK_SIZE};
use crate::rendering::{ShapeCache, TextureCache};
use crate::unit::{Soldier, Worker, WorkerState};

// ── Save file path ──

fn save_dir() -> PathBuf {
    let mut dir = config_dir();
    dir.push("saves");
    dir
}

pub fn save_path() -> PathBuf {
    let mut dir = save_dir();
    dir.push("quicksave.sf_save");
    dir
}

// ── SaveData types (unchanged) ──

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub game_seed: u64,
    pub camera: CameraSave,
    pub wave: WaveSave,
    pub chunk_deposits: HashMap<(i32, i32), Vec<(u32, u32, u32, String)>>,
    pub buildings: Vec<BuildingSave>,
    pub enemies: Vec<EnemySave>,
    pub units: Vec<UnitSave>,
}

#[derive(Serialize, Deserialize)]
pub struct CameraSave { pub x: f32, pub y: f32, pub scale: f32 }

#[derive(Serialize, Deserialize)]
pub struct WaveSave { pub timer: f32, pub wave: u32, pub spawn_timer: f32, pub last_wave: u32 }

#[derive(Serialize, Deserialize)]
pub struct BuildingSave {
    pub kind: String, pub tile_x: i32, pub tile_y: i32,
    pub occupied: Vec<(i32, i32)>,
    pub hp: Option<(u32, u32)>,
    pub inventory: Option<Vec<(String, u32)>>, pub inventory_capacity: u32,
    pub assembler: Option<AssemblerSave>, pub turret: Option<TurretSave>,
    pub belt: Option<BeltSave>, pub storage: bool,
    pub splitter: Option<SplitterSave>, pub sorter: Option<SorterSave>,
}

#[derive(Serialize, Deserialize)]
pub struct AssemblerSave { pub production_timer: f32, pub interval: f32, pub recipe_id: String }

#[derive(Serialize, Deserialize)]
pub struct TurretSave { pub damage: u32, pub range_sq: f32, pub fire_interval: f32, pub timer: f32, pub projectile_speed: f32 }

#[derive(Serialize, Deserialize)]
pub struct BeltSave { pub direction: Direction, pub speed: f32, pub slots: Vec<Option<BeltItemSave>> }

#[derive(Serialize, Deserialize)]
pub struct BeltItemSave { pub resource: String, pub acc: f32 }

#[derive(Serialize, Deserialize)]
pub struct SplitterSave { pub counter: u32, pub outputs: u32, pub input_direction: Option<Direction> }

#[derive(Serialize, Deserialize)]
pub struct SorterSave { pub filter: String, pub inverted: bool }

#[derive(Serialize, Deserialize)]
pub struct EnemySave { pub kind: String, pub x: f32, pub y: f32, pub hp: u32, pub max_hp: u32 }

#[derive(Serialize, Deserialize)]
pub struct UnitSave {
    pub kind: String, pub x: f32, pub y: f32, pub hp: u32, pub max_hp: u32,
    pub soldier_cooldown: Option<f32>, pub worker_timer: Option<f32>,
    pub worker_state: Option<WorkerStateSave>,
}

#[derive(Serialize, Deserialize)]
pub enum WorkerStateSave {
    Idle,
    MovingToDeposit { target_tx: i32, target_ty: i32 },
    Mining { target_tx: i32, target_ty: i32 },
}

// ── Resources ──

#[derive(Resource, Default)]
pub struct SaveManager {
    pub load_requested: Option<String>,
}

#[derive(Resource, Default)]
pub struct ShowPauseMenu(pub bool);

#[derive(Resource, Default)]
pub struct SaveRequested(pub bool);

#[derive(Resource)]
struct LoadBuffer {
    data: Option<SaveData>,
}

impl Default for LoadBuffer {
    fn default() -> Self {
        Self { data: None }
    }
}

pub fn is_fresh_game(fresh: Res<IsFreshGame>) -> bool {
    fresh.0
}

// ── Save system ──

fn save_game(
    keys: Res<ButtonInput<KeyCode>>,
    mut save_req: ResMut<SaveRequested>,
    mut toast: ResMut<ToastQueue>,
    chunk_grid: Res<ChunkGrid>,
    wave: Res<WaveState>,
    last_wave: Res<LastWave>,
    camera: Query<&Transform, With<Camera2d>>,
    tile_positions: Query<&TilePosition>,
    buildings: Query<(
        &Building, &TilePosition, &OccupiedTiles,
        Option<&Health>, Option<&Inventory>,
        Option<&Assembler>, Option<&TurretCombat>,
        Option<&BeltSlots>, Option<&Storage>,
        Option<&Splitter>, Option<&Sorter>,
    )>,
    enemies: Query<(&EnemyComponent, &Transform, &Health, &TilePosition)>,
    units: Query<(&Transform, &Health, &TilePosition, Option<&Soldier>, Option<&Worker>), With<Unit>>,
) {
    if !keys.just_pressed(KeyCode::F5) && !save_req.0 { return; }
    save_req.0 = false;

    let mut data = SaveData {
        version: 1,
        game_seed: chunk_grid.seed(),
        camera: CameraSave { x: 0.0, y: 0.0, scale: 1.0 },
        wave: WaveSave {
            timer: wave.timer, wave: wave.wave,
            spawn_timer: wave.spawn_timer, last_wave: last_wave.0,
        },
        chunk_deposits: HashMap::new(),
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
            data.chunk_deposits.insert((*cx, *cy), chunk.deposits.clone());
        }
    }

    for (building, pos, occupied, hp, inventory, assembler,
         turret, belt, storage, splitter, sorter) in buildings.iter() {
        let belt_save = belt.map(|b| {
            let slots: Vec<Option<BeltItemSave>> = b.items.iter().map(|item| {
                item.as_ref().map(|i| BeltItemSave {
                    resource: i.resource_id.0.clone(), acc: i.acc,
                })
            }).collect();
            BeltSave { direction: b.direction, speed: b.speed, slots }
        });
        data.buildings.push(BuildingSave {
            kind: building.kind.clone(),
            tile_x: pos.x, tile_y: pos.y,
            occupied: occupied.0.clone(),
            hp: hp.map(|h| (h.current, h.max)),
            inventory: inventory.map(|inv| inv.resources.iter().map(|(r, a)| (r.0.clone(), *a)).collect()),
            inventory_capacity: inventory.map(|inv| inv.capacity).unwrap_or(0),
            assembler: assembler.map(|a| AssemblerSave { production_timer: a.production_timer, interval: a.interval, recipe_id: a.recipe_id.clone() }),
            turret: turret.map(|t| TurretSave { damage: t.damage, range_sq: t.range_sq, fire_interval: t.fire_interval, timer: t.timer, projectile_speed: t.projectile_speed }),
            belt: belt_save, storage: storage.is_some(),
            splitter: splitter.map(|s| SplitterSave { counter: s.counter, outputs: s.outputs, input_direction: s.input_direction }),
            sorter: sorter.map(|s| SorterSave { filter: s.filter.0.clone(), inverted: s.inverted }),
        });
    }

    for (enemy, tf, hp, _) in enemies.iter() {
        data.enemies.push(EnemySave {
            kind: enemy.kind.clone(),
            x: tf.translation.x, y: tf.translation.y,
            hp: hp.current, max_hp: hp.max,
        });
    }

    for (tf, hp, _pos, soldier, worker) in units.iter() {
        let kind = if soldier.is_some() { "soldier" } else { "worker" };
        let worker_state = worker.map(|w| match &w.state {
            WorkerState::Idle => WorkerStateSave::Idle,
            WorkerState::MovingToDeposit(e) => {
                tile_positions.get(*e).map(|pos| WorkerStateSave::MovingToDeposit {
                    target_tx: pos.x, target_ty: pos.y
                }).unwrap_or(WorkerStateSave::Idle)
            }
            WorkerState::Mining(e) => {
                tile_positions.get(*e).map(|pos| WorkerStateSave::Mining {
                    target_tx: pos.x, target_ty: pos.y
                }).unwrap_or(WorkerStateSave::Idle)
            }
        });
        data.units.push(UnitSave {
            kind: kind.to_string(), x: tf.translation.x, y: tf.translation.y,
            hp: hp.current, max_hp: hp.max,
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

// ── Cleanup world ──

fn cleanup_world(
    mut commands: Commands,
    buildings: Query<Entity, With<Building>>,
    enemies: Query<Entity, With<EnemyComponent>>,
    units: Query<Entity, With<Unit>>,
    deposits: Query<Entity, With<ResourceDeposit>>,
    markers: Query<Entity, With<ChunkMarker>>,
    members: Query<Entity, (With<ChunkMember>, Without<ResourceDeposit>)>,
    cameras: Query<Entity, With<Camera2d>>,
    belt_slots: Query<&BeltSlots>,
    ghosts: Query<Entity, With<Ghost>>,
    hp_bars: Query<Entity, With<HpBarChild>>,
    menus: Query<Entity, With<crate::economy::components::MenuBarPanel>>,
    popups: Query<Entity, With<PanelModal>>,
    ui_texts: Query<Entity, With<ResourceCountText>>,
    pause_menus: Query<Entity, With<PauseMenuRoot>>,
    projectiles: Query<Entity, With<Projectile>>,
) {
    for bs in belt_slots.iter() {
        for sprite_entity in bs.slot_sprites.iter().flatten() {
            silent_despawn(&mut commands, *sprite_entity);
        }
    }
    for e in buildings.iter().chain(enemies.iter()).chain(units.iter())
        .chain(deposits.iter()).chain(markers.iter()).chain(members.iter())
        .chain(cameras.iter()).chain(ghosts.iter())
        .chain(hp_bars.iter()).chain(menus.iter()).chain(popups.iter())
        .chain(ui_texts.iter()).chain(pause_menus.iter()).chain(projectiles.iter())
    {
        silent_despawn(&mut commands, e);
    }
}

// ── Load chain (runs on OnEnter(Loading)) ──

fn read_save_file(
    mut save_mgr: ResMut<SaveManager>,
    mut buf: ResMut<LoadBuffer>,
    mut toast: ResMut<ToastQueue>,
) {
    let path = match &save_mgr.load_requested {
        Some(p) => PathBuf::from(p),
        None => return,
    };
    *save_mgr = SaveManager { load_requested: None };
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => { toast.0.push(format!("Load failed: {e}")); return; }
    };
    let data: SaveData = match ron::from_str(&content) {
        Ok(d) => d,
        Err(e) => { toast.0.push(format!("Save file corrupt: {e}")); return; }
    };
    buf.data = Some(data);
}

fn load_chunks(
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
    let data = match &buf.data { Some(d) => d, None => return };
    chunk_grid.clear();
    chunk_grid.set_seed(data.game_seed);
    for ((cx, cy), deposits) in &data.chunk_deposits {
        let chunk = chunk_grid.ensure_chunk_mut(*cx, *cy);
        chunk.deposits = deposits.clone();
    }

    let (hx, hy) = data.buildings.iter().find(|b| b.kind == "hq")
        .map(|b| (b.tile_x, b.tile_y)).unwrap_or((0, 0));
    let chunk_size = CHUNK_SIZE as i32;
    let hq_cx = hx.div_euclid(chunk_size);
    let hq_cy = hy.div_euclid(chunk_size);

    for cx in (hq_cx - 10)..=(hq_cx + 10) {
        for cy in (hq_cy - 10)..=(hq_cy + 10) {
            spawn_single_chunk_visuals(&mut commands, &mut chunk_grid, &cfg, &res_registry, &shapes, &mut materials, &mut meshes, &textures, cx, cy);
        }
    }
}

fn spawn_fresh_camera(
    mut commands: Commands,
    cfg: Res<MapConfig>,
    buf: Res<LoadBuffer>,
) {
    if buf.data.is_some() { return; }
    let (hx, hy) = cfg.hq_position;
    commands.spawn((
        Camera2d,
        bevy::ui::IsDefaultUiCamera,
        Transform::from_xyz(
            hx as f32 * cfg.tile_size + cfg.tile_size / 2.0,
            hy as f32 * cfg.tile_size + cfg.tile_size / 2.0,
            100.0,
        ),
        bevy_pancam::PanCam {
            grab_buttons: vec![MouseButton::Middle],
            speed: 500.0,
            min_scale: 0.3,
            max_scale: 3.0,
            ..default()
        },
    ));
}

fn load_camera(
    buf: Res<LoadBuffer>,
    mut commands: Commands,
) {
    let data = match &buf.data { Some(d) => d, None => return };
    commands.spawn((
        Camera2d,
        bevy::ui::IsDefaultUiCamera,
        Transform::from_xyz(data.camera.x, data.camera.y, 100.0).with_scale(Vec3::splat(data.camera.scale)),
        bevy_pancam::PanCam { grab_buttons: vec![MouseButton::Middle], speed: 500.0, min_scale: 0.3, max_scale: 3.0, ..default() },
    ));
}

fn load_buildings(
    buf: Res<LoadBuffer>,
    mut commands: Commands,
    cfg: Res<MapConfig>,
) {
    let data = match &buf.data { Some(d) => d, None => return };
    let tile_size = cfg.tile_size;

    for bs in &data.buildings {
        let (tw, th) = if bs.kind == "hq" { (2, 2) } else { (1, 1) };
        let cx = (bs.tile_x as f32 + (tw as f32 - 1.0) * 0.5) * tile_size;
        let cy = (bs.tile_y as f32 + (th as f32 - 1.0) * 0.5) * tile_size;
        let inv = if let Some(ref items) = bs.inventory {
            let mut i = Inventory::with_capacity(bs.inventory_capacity);
            for (res, amount) in items { i.add(&ResourceId(res.clone()), *amount); }
            i
        } else if bs.inventory_capacity > 0 { Inventory::with_capacity(bs.inventory_capacity) }
        else { Inventory::new() };
        let tf = Transform::from_xyz(cx, cy, 2.0);
        let tile_pos = TilePosition { x: bs.tile_x, y: bs.tile_y };
        let occupied = OccupiedTiles(bs.occupied.clone());
        let building = Building { kind: bs.kind.clone(), name: bs.kind.clone() };

        if bs.kind == "hq" {
            commands.spawn((
                HQ, building, inv, occupied, tf, tile_pos, Active(true),
            ));
        } else if bs.kind == "miner" {
            let a = bs.assembler.as_ref().unwrap();
            commands.spawn((
                Miner,
                Assembler { production_timer: a.production_timer, interval: a.interval, recipe_id: a.recipe_id.clone() },
                building, inv, occupied, tf, tile_pos, Active(true),
            ));
        } else if bs.kind == "assembler" || bs.kind == "furnace" {
            let a = bs.assembler.as_ref().unwrap();
            commands.spawn((
                Assembler { production_timer: a.production_timer, interval: a.interval, recipe_id: a.recipe_id.clone() },
                building, inv, occupied, tf, tile_pos, Active(true),
            ));
        } else if bs.belt.is_some() || bs.splitter.is_some() || bs.sorter.is_some() {
            let b = bs.belt.as_ref().unwrap();
            let slot_positions = crate::economy::belt::compute_slot_positions(
                bs.tile_x, bs.tile_y, b.direction, b.slots.len() as u32, tile_size);
            let angle = match b.direction {
                Direction::East => 0.0,
                Direction::North => std::f32::consts::FRAC_PI_2,
                Direction::West => std::f32::consts::PI,
                Direction::South => -std::f32::consts::FRAC_PI_2,
            };
            let belt_tf = Transform::from_xyz(cx, cy, 2.0).with_rotation(Quat::from_rotation_z(angle));
            let mut items: Vec<Option<ItemOnBelt>> = Vec::new();
            for item_save in &b.slots {
                if let Some(item) = item_save {
                    items.push(Some(ItemOnBelt { resource_id: ResourceId(item.resource.clone()), acc: item.acc }));
                } else { items.push(None); }
            }
            let slot_sprites: Vec<Option<Entity>> = vec![None; items.len()];
            let belt_comp = BeltSlots { direction: b.direction, items, slot_sprites, slot_positions, speed: b.speed };
            if let Some(sp) = &bs.splitter {
                commands.spawn((
                    belt_comp, building, inv, occupied, belt_tf, tile_pos,
                    Splitter { counter: sp.counter, outputs: sp.outputs, input_direction: sp.input_direction },
                    Active(true),
                ));
            } else if let Some(so) = &bs.sorter {
                commands.spawn((
                    belt_comp, building, inv, occupied, belt_tf, tile_pos,
                    Sorter { filter: ResourceId(so.filter.clone()), inverted: so.inverted },
                    Active(true),
                ));
            } else {
                commands.spawn((
                    belt_comp, building, inv, occupied, belt_tf, tile_pos,
                    Active(true),
                ));
            }
        } else if bs.kind == "turret" {
            let t = bs.turret.as_ref().unwrap();
            commands.spawn((
                TurretCombat { damage: t.damage, range_sq: t.range_sq, fire_interval: t.fire_interval, timer: t.timer, projectile_speed: t.projectile_speed },
                building, inv, occupied, tf, tile_pos, Active(true),
            ));
        } else if bs.storage {
            commands.spawn((
                Storage, building, inv, occupied, tf, tile_pos, Active(true),
            ));
        } else {
            commands.spawn((
                building, inv, occupied, tf, tile_pos, Active(true),
            ));
        }
    }
}

fn load_enemies(
    buf: Res<LoadBuffer>,
    mut commands: Commands,
    cfg: Res<MapConfig>,
) {
    let data = match &buf.data { Some(d) => d, None => return };
    for es in &data.enemies {
        let entity = commands.spawn((
            EnemyComponent { kind: es.kind.clone() }, Health { current: es.hp, max: es.max_hp },
            Transform::from_xyz(es.x, es.y, 3.0),
            TilePosition { x: (es.x / cfg.tile_size) as i32, y: (es.y / cfg.tile_size) as i32 },
        )).id();
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                HpBarChild,
                Sprite { custom_size: Some(Vec2::new(24.0, 3.0)), color: Color::srgb(0.2, 1.0, 0.2), ..default() },
                Transform::from_xyz(0.0, 20.0, 1.0),
            ));
        });
    }
}

fn load_units(
    buf: Res<LoadBuffer>,
    mut commands: Commands,
    cfg: Res<MapConfig>,
) {
    let data = match &buf.data { Some(d) => d, None => return };
    for us in &data.units {
        if us.kind == "worker" {
            commands.spawn((
                Worker { state: WorkerState::Idle, mining_timer: us.worker_timer.unwrap_or(0.0) },
                Unit, Health { current: us.hp, max: us.max_hp },
                Transform::from_xyz(us.x, us.y, 2.5),
                TilePosition { x: (us.x / cfg.tile_size) as i32, y: (us.y / cfg.tile_size) as i32 },
            ));
        } else {
            commands.spawn((
                Soldier { attack_cooldown: us.soldier_cooldown.unwrap_or(0.0) },
                Unit, Health { current: us.hp, max: us.max_hp },
                Transform::from_xyz(us.x, us.y, 2.5),
                TilePosition { x: (us.x / cfg.tile_size) as i32, y: (us.y / cfg.tile_size) as i32 },
            ));
        }
    }
}

fn load_finalize(
    mut buf: ResMut<LoadBuffer>,
    mut wave: ResMut<WaveState>,
    mut last_wave: ResMut<LastWave>,
    mut peaceful: ResMut<PeacefulMode>,
    mut fresh_game: ResMut<IsFreshGame>,
    mut next_state: ResMut<NextState<GameState>>,
    mut toast: ResMut<ToastQueue>,
) {
    let data = match &buf.data { Some(d) => d, None => {
        buf.data = None;
        next_state.set(GameState::Menu);
        return;
    }};
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

// ── Pause menu ──

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
struct SaveButton;

#[derive(Component)]
struct LoadButton;

#[derive(Component)]
struct ResumeButton;

#[derive(Component)]
struct QuitButton;

fn toggle_pause_menu(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<crate::core::input::KeyBindings>,
    mut show: ResMut<ShowPauseMenu>,
) {
    if keys.just_pressed(bindings.key("cancel")) {
        show.0 = !show.0;
    }
}

fn spawn_pause_menu(
    mut commands: Commands,
    show: Res<ShowPauseMenu>,
    panel_query: Query<Entity, With<PauseMenuRoot>>,
) {
    if show.0 && panel_query.is_empty() {
        let _ = commands.spawn((
            PauseMenuRoot,
            Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
                display: Display::Flex, flex_direction: FlexDirection::Column, align_items: AlignItems::Center,
                justify_content: JustifyContent::Center, ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            Pickable::default(),
        )).with_children(|parent| {
            parent.spawn((
                Node { display: Display::Flex, flex_direction: FlexDirection::Column, align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(24.0)), row_gap: Val::Px(8.0), ..default() },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9)),
                Outline { width: Val::Px(2.0), offset: Val::ZERO, color: Color::srgb(0.4, 0.4, 0.5) },
            )).with_children(|panel| {
                panel.spawn((Text::new("PAUSED"), TextFont::from_font_size(28.0), TextColor(Color::srgb(0.8, 0.8, 1.0)),
                    Node { margin: UiRect::bottom(Val::Px(12.0)), ..default() }));
                // Save button
                panel.spawn((
                    SaveButton, Button,
                    Node { width: Val::Px(200.0), height: Val::Px(40.0), align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center, ..default() },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                )).with_children(|btn| {
                    btn.spawn((Text::new("Save Game"), TextFont::from_font_size(16.0), TextColor(Color::WHITE)));
                });
                // Load button
                panel.spawn((
                    LoadButton, Button,
                    Node { width: Val::Px(200.0), height: Val::Px(40.0), align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center, ..default() },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                )).with_children(|btn| {
                    btn.spawn((Text::new("Load Game"), TextFont::from_font_size(16.0), TextColor(Color::WHITE)));
                });
                // Resume button
                panel.spawn((
                    ResumeButton, Button,
                    Node { width: Val::Px(200.0), height: Val::Px(40.0), align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center, ..default() },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                )).with_children(|btn| {
                    btn.spawn((Text::new("Resume"), TextFont::from_font_size(16.0), TextColor(Color::WHITE)));
                });
                // Quit button
                panel.spawn((
                    QuitButton, Button,
                    Node { width: Val::Px(200.0), height: Val::Px(40.0), align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center, ..default() },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                )).with_children(|btn| {
                    btn.spawn((Text::new("Main Menu"), TextFont::from_font_size(16.0), TextColor(Color::WHITE)));
                });
            });
        });
    } else if !show.0 {
        for entity in &panel_query {
            silent_despawn(&mut commands, entity);
        }
    }
}

fn resume_interaction(
    query: Query<&Interaction, (Changed<Interaction>, With<ResumeButton>)>,
    mut show: ResMut<ShowPauseMenu>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed { show.0 = false; }
    }
}

fn quit_interaction(
    query: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut show: ResMut<ShowPauseMenu>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            show.0 = false;
            next_state.set(GameState::Menu);
        }
    }
}

fn save_interaction(
    query: Query<&Interaction, (Changed<Interaction>, With<SaveButton>)>,
    mut show: ResMut<ShowPauseMenu>,
    mut save_req: ResMut<SaveRequested>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            show.0 = false;
            save_req.0 = true;
        }
    }
}

fn load_interaction(
    query: Query<&Interaction, (Changed<Interaction>, With<LoadButton>)>,
    mut save_mgr: ResMut<SaveManager>,
    mut next_state: ResMut<NextState<GameState>>,
    mut show: ResMut<ShowPauseMenu>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            show.0 = false;
            save_mgr.load_requested = Some(save_path().to_string_lossy().to_string());
            next_state.set(GameState::Loading);
        }
    }
}

fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuRoot>>) {
    for e in &query {
        silent_despawn(&mut commands, e);
    }
}

// ── Plugin ──

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SaveManager::default());
        app.insert_resource(ShowPauseMenu(false));
        app.insert_resource(SaveRequested(false));
        app.insert_resource(LoadBuffer::default());
        app.insert_resource(IsFreshGame(true));

        // Loading: clean slate then rebuild from save file
        app.add_systems(OnEnter(GameState::Loading), (
            cleanup_world,
            read_save_file,
            load_chunks,
            load_camera,
            load_buildings,
            load_enemies,
            load_units,
            load_finalize,
        ).chain());

        // Fresh game: spawn camera at HQ position
        app.add_systems(OnEnter(GameState::Playing),
            spawn_fresh_camera.run_if(is_fresh_game));
        app.add_systems(OnExit(GameState::Playing), cleanup_pause_menu);

        app.add_systems(Update, (
            save_game,
            toggle_pause_menu,
            spawn_pause_menu,
            resume_interaction,
            quit_interaction,
            save_interaction,
            load_interaction,
        ).run_if(in_state(GameState::Playing)));
    }
}
