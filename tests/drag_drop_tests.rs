use bevy::prelude::*;
use siege_factory::economy::components::{DragState, InventoryGrid, InventorySlot, Player};
use siege_factory::economy::resource::{Inventory, ResourceId};

/// drag_start fires when a slot has Interaction::Pressed and contains a resource.
#[test]
fn test_drag_start_on_press() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.init_resource::<DragState>();
    app.add_systems(Update, siege_factory::ui::components::inventory_drag::drag_start);

    let mut inv = Inventory::with_slots(1, 0);
    inv.add(&ResourceId("ore".into()), 5);
    let inv_entity = app.world_mut().spawn(inv).id();

    let slot = app
        .world_mut()
        .spawn((
            InventorySlot { index: 0 },
            Interaction::Pressed,
        ))
        .id();
    let grid = app
        .world_mut()
        .spawn(InventoryGrid {
            cols: 5,
            rows: 1,
            owner: inv_entity,
        })
        .id();
    app.world_mut().entity_mut(grid).add_child(slot);

    app.update();

    let drag = app.world().resource::<DragState>();
    assert!(drag.active, "drag_start should activate on Interaction::Pressed");
    assert_eq!(drag.source_owner, Some(inv_entity));
    assert_eq!(drag.source_slot_index, 0);
    assert_eq!(drag.amount, 5, "entire stack (5) is picked up by default");
}

/// drag_end transfers 1 unit from source to target when target has Interaction::Hovered.
#[test]
fn test_drag_end_transfer_works() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<DragState>();
    app.insert_resource(ButtonInput::<MouseButton>::default());
    app.insert_resource(siege_factory::core::toast::ToastQueue(Vec::new()));
    app.add_systems(Update, siege_factory::ui::components::inventory_drag::drag_end);

    let mut src_inv = Inventory::with_slots(1, 0);
    src_inv.add(&ResourceId("iron_ore".into()), 5);
    let src = app.world_mut().spawn(src_inv).id();
    let dst = app.world_mut().spawn(Inventory::new()).id();

    let slot = app
        .world_mut()
        .spawn((
            InventorySlot { index: 0 },
            Interaction::Hovered,
        ))
        .id();
    let grid = app
        .world_mut()
        .spawn(InventoryGrid {
            cols: 1,
            rows: 1,
            owner: dst,
        })
        .id();
    app.world_mut().entity_mut(grid).add_child(slot);

    let mut d = app.world_mut().resource_mut::<DragState>();
    d.active = true;
    d.source_owner = Some(src);
    d.source_slot_index = 0;
    d.resource = Some(ResourceId("iron_ore".into()));
    d.amount = 1;

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();

    let s = app.world().get::<Inventory>(src).unwrap();
    let d = app.world().get::<Inventory>(dst).unwrap();
    assert_eq!(s.get(&ResourceId("iron_ore".into())), 4, "source loses 1");
    assert_eq!(d.get(&ResourceId("iron_ore".into())), 1, "target gains 1");
}

/// Same-owner swap via drag_end when target slot has Interaction::Hovered.
#[test]
fn test_same_inventory_swap() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<DragState>();
    app.insert_resource(ButtonInput::<MouseButton>::default());
    app.insert_resource(siege_factory::core::toast::ToastQueue(Vec::new()));
    app.add_systems(Update, siege_factory::ui::components::inventory_drag::drag_end);

    let mut inv = Inventory::with_slots(2, 0);
    inv.add(&ResourceId("ore".into()), 5);
    inv.add(&ResourceId("iron_ore".into()), 3);
    let inv_entity = app.world_mut().spawn(inv).id();

    let slot = app
        .world_mut()
        .spawn((
            InventorySlot { index: 1 },
            Interaction::Hovered,
        ))
        .id();
    let grid = app
        .world_mut()
        .spawn(InventoryGrid {
            cols: 2,
            rows: 1,
            owner: inv_entity,
        })
        .id();
    app.world_mut().entity_mut(grid).add_child(slot);

    let mut d = app.world_mut().resource_mut::<DragState>();
    d.active = true;
    d.source_owner = Some(inv_entity);
    d.source_slot_index = 0;
    d.resource = Some(ResourceId("ore".into()));
    d.amount = 1;

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();

    let result = app.world().get::<Inventory>(inv_entity).unwrap();
    assert_eq!(
        result.slot_content(0).map(|(r, _)| r.0.as_str()),
        Some("iron_ore"),
        "slot 0 should contain iron_ore after swap"
    );
    assert_eq!(
        result.slot_content(1).map(|(r, _)| r.0.as_str()),
        Some("ore"),
        "slot 1 should contain ore after swap"
    );
}

/// drag_start must NOT fire when slot has no Interaction or Interaction != Pressed.
#[test]
fn test_drag_start_no_press() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.init_resource::<DragState>();
    app.add_systems(Update, siege_factory::ui::components::inventory_drag::drag_start);

    let mut inv = Inventory::with_slots(1, 0);
    inv.add(&ResourceId("ore".into()), 5);
    let _inv_entity = app.world_mut().spawn(inv).id();

    // Slot without Interaction → Changed<Interaction> filter won't match
    let _slot = app
        .world_mut()
        .spawn(InventorySlot { index: 0 })
        .id();

    app.update();
    let drag = app.world().resource::<DragState>();
    assert!(!drag.active, "drag must NOT start without Interaction::Pressed");
}

/// After opening the inventory (via direct spawn), inserting Interaction::Pressed on a slot starts the drag.
#[test]
fn test_drag_after_inventory_open() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.init_resource::<DragState>();
    app.add_systems(Update, siege_factory::ui::components::inventory_drag::drag_start);

    let mut inv = Inventory::with_slots(20, 0);
    inv.add(&ResourceId("ore".into()), 5);
    let player = app.world_mut().spawn((Player, inv)).id();

    // Directly spawn an inventory grid + slots (bypass TOML for test simplicity)
    use siege_factory::economy::components::InventoryGrid;
    use siege_factory::economy::components::InventorySlot;
    let grid = app.world_mut().spawn((
        InventoryGrid { cols: 5, rows: 4, owner: player },
        Node::default(),
    )).id();
    for i in 0..20 {
        let slot = app.world_mut().spawn((
            InventorySlot { index: i },
            Button,
            Node::default(),
        )).id();
        app.world_mut().entity_mut(grid).add_child(slot);
    }

    app.update();

    // Simulate ui_focus_system: mark slot 0 as pressed
    let slot_ids: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<InventorySlot>>()
        .iter(app.world())
        .take(1)
        .collect();
    if let Some(&slot_entity) = slot_ids.first() {
        app.world_mut()
            .entity_mut(slot_entity)
            .insert(Interaction::Pressed);
    }

    app.update();

    let drag = app.world().resource::<DragState>();
    assert!(drag.active, "drag must start when a slot has Interaction::Pressed after inventory open");
    assert_eq!(drag.source_slot_index, 0);
}

// test_ui_gpu removed: needs real GPU + WinitPlugin (main thread).
// The drag logic is fully covered by the non-GPU tests above.
