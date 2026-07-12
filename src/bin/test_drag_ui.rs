//! Test: reproduces the user's drag-and-drop bug.
//!
//! Run with: cargo run --bin test_drag_ui
//!
//! Uses MinimalPlugins + UiPlugin (no GPU window needed).
//! Opens inventory, clicks on slot, verifies drag starts.

use bevy::ecs::message::Messages;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use bevy::window::PrimaryWindow;
use bevy::input::touch::Touches;
use siege_factory::economy::components::{DragState, InventorySlot, Player};
use siege_factory::economy::resource::{Inventory, ResourceId};
use siege_factory::ui::components::inventory_drag::drag_start;
use siege_factory::ui::global_panels::toggle_inventory;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(TransformPlugin);
    app.add_plugins(bevy::input::InputPlugin);
    app.add_plugins(bevy::window::WindowPlugin::default());
    app.add_plugins(bevy::ui::UiPlugin);
    app.init_resource::<Touches>();

    app.init_resource::<DragState>();
    app.add_systems(Update, (toggle_inventory, drag_start));

    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app.world_mut().spawn(Camera2d);

    let mut inv = Inventory::with_slots(20, 0);
    inv.add(&ResourceId("ore".into()), 5);
    inv.add(&ResourceId("iron_ore".into()), 3);
    app.world_mut().spawn((Player, inv));

    let window_entity = app
        .world_mut()
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .iter(app.world())
        .next()
        .expect("no primary window");

    app.world_mut()
        .resource_mut::<Messages<KeyboardInput>>()
        .write(KeyboardInput {
            key_code: KeyCode::KeyI,
            logical_key: Key::Character("i".into()),
            text: Some("i".into()),
            state: ButtonState::Pressed,
            window: window_entity,
            repeat: false,
        });

    for _ in 0..10 {
        app.update();
    }

    let slot_0_pos = app
        .world_mut()
        .query::<(&InventorySlot, &UiGlobalTransform)>()
        .iter(app.world())
        .find_map(|(slot, gt)| {
            if slot.index == 0 {
                Some(gt.translation)
            } else {
                None
            }
        })
        .expect("FAIL 1: slot 0 has NO UiGlobalTransform after layout");

    println!("slot 0 position after layout: {slot_0_pos:?}");

    app.world_mut()
        .query_filtered::<&mut Window, With<PrimaryWindow>>()
        .single_mut(app.world_mut())
        .expect("window")
        .set_cursor_position(Some(slot_0_pos));

    app.world_mut()
        .resource_mut::<Messages<MouseButtonInput>>()
        .write(MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Pressed,
            window: window_entity,
        });

    app.update();

    let drag = app.world().resource::<DragState>();
    if drag.active {
        println!("=== PASS: drag started at {:?} ===", slot_0_pos);
    } else {
        println!("=== FAIL: clicked slot 0 at {:?} but drag did NOT start ===", slot_0_pos);
        std::process::exit(1);
    }
}
