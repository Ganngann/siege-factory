pub mod agriculture;
pub mod combat;
pub mod core;
pub mod economy;
pub mod enemy;
pub mod events;
pub mod map;
pub mod player;
pub mod rendering;
pub mod save_load;
pub mod unit;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_pancam::PanCamPlugin;
use combat::CombatPlugin;
use core::schedule::{CorePlugin, GameplayStep};
use economy::EconomyPlugin;
use events::CleanupPlugin;
use map::systems::MapPlugin;
use rendering::RenderPlugin;
use save_load::SaveLoadPlugin;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanCamPlugin::default())
        .add_plugins(core::modding::ModPlugin)
        .add_plugins(CorePlugin)
        .add_plugins(MapPlugin)
        .add_plugins(EconomyPlugin)
        .add_plugins(enemy::EnemyPlugin)
        .add_plugins(unit::UnitPlugin)
        .add_plugins(agriculture::AgriculturePlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(RenderPlugin)
        .add_plugins(CleanupPlugin)
        .add_plugins(SaveLoadPlugin)
        .insert_resource(WinitSettings::desktop_app())
        .configure_sets(Update, (
            GameplayStep::PlayerInput,
            GameplayStep::CameraFollow,
            GameplayStep::ChunkManagement,
            GameplayStep::FogOfWar,
        ).chain())
        .run();
}
