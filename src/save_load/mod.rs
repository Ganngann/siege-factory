use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::game_state::{GameState, IsFreshGame};
use crate::core::utils::config_dir;

pub mod load;
pub mod pause_menu;
pub mod save;

pub use load::*;
pub use pause_menu::*;
pub use save::*;

// ── load_data! macro (used by load.rs) ──

macro_rules! load_data {
    ($buf:expr) => {
        match &$buf.data {
            Some(d) => d,
            None => return,
        }
    };
}
pub(crate) use load_data;

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

// ── SaveData types ──

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub game_seed: u64,
    pub camera: CameraSave,
    pub wave: WaveSave,
    pub chunk_deposits: HashMap<(i32, i32), Vec<crate::map::tile_grid::Deposit>>,
    pub visited: HashMap<(i32, i32), Vec<(u32, u32)>>,
    pub buildings: Vec<BuildingSave>,
    pub enemies: Vec<EnemySave>,
    pub units: Vec<UnitSave>,
    #[serde(default)]
    pub tutorial: TutorialSave,
}

#[derive(Serialize, Deserialize, Default)]
pub struct TutorialSave {
    pub current_index: usize,
    pub completed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct CameraSave {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
}

#[derive(Serialize, Deserialize)]
pub struct WaveSave {
    pub timer: f32,
    pub wave: u32,
    pub spawn_timer: f32,
    pub last_wave: u32,
}

#[derive(Serialize, Deserialize)]
pub struct FarmSave {
    pub crop_types: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct BuildingSave {
    pub kind: String,
    pub tile_x: i32,
    pub tile_y: i32,
    pub occupied: Vec<(i32, i32)>,
    pub hp: Option<(u32, u32)>,
    pub inventory: Option<Vec<(String, u32)>>,
    pub inventory_capacity: u32,
    pub assembler: Option<AssemblerSave>,
    pub turret: Option<TurretSave>,
    pub belt: Option<BeltSave>,
    pub storage: bool,
    pub splitter: Option<SplitterSave>,
    pub sorter: Option<SorterSave>,
    pub farm: Option<FarmSave>,
    pub power_draw: Option<f32>,
    #[serde(default)]
    pub power_generation: f32,
    #[serde(default)]
    pub power_pole_range: f32,
}

#[derive(Serialize, Deserialize)]
pub struct AssemblerSave {
    pub production_timer: f32,
    pub interval: f32,
    pub recipe_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct TurretSave {
    pub damage: u32,
    pub range_sq: f32,
    pub fire_interval: f32,
    pub timer: f32,
    pub projectile_speed: f32,
}

#[derive(Serialize, Deserialize)]
pub struct BeltSave {
    pub direction: crate::economy::components::Direction,
    pub speed: f32,
    pub slots: Vec<Option<BeltItemSave>>,
}

#[derive(Serialize, Deserialize)]
pub struct BeltItemSave {
    pub resource: String,
    pub acc: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SplitterSave {
    pub counter: u32,
    pub outputs: u32,
    pub input_direction: Option<crate::economy::components::Direction>,
}

#[derive(Serialize, Deserialize)]
pub struct SorterSave {
    pub filter: String,
    pub inverted: bool,
}

#[derive(Serialize, Deserialize)]
pub struct EnemySave {
    pub kind: String,
    pub x: f32,
    pub y: f32,
    pub hp: u32,
    pub max_hp: u32,
}

#[derive(Serialize, Deserialize)]
pub struct UnitSave {
    pub kind: String,
    pub x: f32,
    pub y: f32,
    pub hp: u32,
    pub max_hp: u32,
    pub soldier_cooldown: Option<f32>,
    pub worker_timer: Option<f32>,
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
pub struct LoadBuffer {
    pub data: Option<SaveData>,
}

impl Default for LoadBuffer {
    fn default() -> Self {
        Self { data: None }
    }
}

pub fn is_fresh_game(fresh: Res<IsFreshGame>) -> bool {
    fresh.0
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
        app.add_systems(
            OnEnter(GameState::Loading),
            (
                pause_menu::cleanup_world,
                load::read_save_file,
                load::load_chunks,
                load::load_camera,
                load::load_buildings,
                load::load_enemies,
                load::load_units,
                load::load_finalize,
            )
                .chain(),
        );

        // Fresh game: spawn camera at player position
        app.add_systems(
            OnEnter(GameState::Playing),
            spawn_fresh_camera.run_if(is_fresh_game),
        );
        app.add_systems(OnExit(GameState::Playing), pause_menu::cleanup_pause_menu);

        app.add_systems(
            Update,
            (
                save::save_game,
                pause_menu::toggle_pause_menu,
                pause_menu::spawn_pause_menu,
                pause_menu::resume_interaction,
                pause_menu::quit_interaction,
                pause_menu::save_interaction,
                pause_menu::load_interaction,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
