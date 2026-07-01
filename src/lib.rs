pub mod combat;
pub mod core;
pub mod economy;
pub mod enemy;
pub mod events;
pub mod map;
pub mod rendering;
pub mod unit;

use bevy::prelude::*;
use combat::CombatPlugin;
use core::schedule::CorePlugin;
use economy::EconomyPlugin;
use events::CleanupPlugin;
use map::systems::MapPlugin;
use rendering::RenderPlugin;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CorePlugin)
        .add_plugins(MapPlugin)
        .add_plugins(EconomyPlugin)
        .add_plugins(enemy::EnemyPlugin)
        .add_plugins(unit::UnitPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(RenderPlugin)
        .add_plugins(CleanupPlugin)
        .run();
}
