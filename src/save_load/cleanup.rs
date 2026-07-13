#![allow(clippy::type_complexity)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::drop_non_drop)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::useless_format)]
#![allow(clippy::single_match)]
use bevy::prelude::*;

use crate::combat::Projectile;
use crate::economy::belt::BeltSlots;
use crate::economy::components::{
    Builder, Building, Ghost, HpBarChild, MenuBarPanel, PanelOverlay, Player, ResourceDeposit, Unit,
};
use crate::core::utils::silent_despawn;
use crate::enemy::components::Enemy as EnemyComponent;
use crate::map::components::ChunkMember;
use crate::map::systems::ChunkMarker;
use crate::ui::components::inventory_drag::InventoryPanel;
use crate::ui::components::pause_menu::PauseMenuRoot;

pub fn cleanup_world(
    mut commands: Commands,
    belt_slots: Query<&BeltSlots>,
    to_despawn: Query<
        Entity,
        Or<(
            With<Building>,
            With<EnemyComponent>,
            With<Unit>,
            With<ResourceDeposit>,
            With<ChunkMarker>,
            With<ChunkMember>,
            With<Camera2d>,
            With<Ghost>,
            With<HpBarChild>,
            With<MenuBarPanel>,
            With<PanelOverlay>,
            With<InventoryPanel>,
            With<PauseMenuRoot>,
            With<Projectile>,
        )>,
    >,
    player_builder: Query<Entity, Or<(With<Player>, With<Builder>)>>,
) {
    for bs in belt_slots.iter() {
        for sprite_entity in bs.slot_sprites.iter().flatten() {
            silent_despawn(&mut commands, *sprite_entity);
        }
    }
    for e in &to_despawn {
        silent_despawn(&mut commands, e);
    }
    for e in &player_builder {
        silent_despawn(&mut commands, e);
    }
}
