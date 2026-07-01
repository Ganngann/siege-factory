pub mod core;
pub mod map;

use bevy::prelude::*;
use core::schedule::CorePlugin;
use map::systems::MapPlugin;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CorePlugin)
        .add_plugins(MapPlugin)
        .run();
}
