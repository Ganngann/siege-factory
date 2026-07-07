use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use siege_factory::economy::components::{DragState, InventoryGrid, InventorySlot};
use siege_factory::economy::resource::{Inventory, ResourceId};

#[test]
fn test_rect_hittest_basic() {
    let center = Vec2::new(200.0, 300.0);
    let size = Vec2::splat(48.0);
    let rect = Rect::from_center_size(center, size);
    assert!(rect.contains(Vec2::new(200.0, 300.0)));
    assert!(rect.contains(Vec2::new(176.0, 276.0)));
    assert!(rect.contains(Vec2::new(224.0, 324.0)));
    assert!(!rect.contains(Vec2::new(100.0, 100.0)));
}

#[test]
fn test_drag_start_via_interaction() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<DragState>();
    app.add_systems(Update, siege_factory::economy::ui::drag_start);

    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let mut w = app
        .world_mut()
        .query_filtered::<&mut Window, With<PrimaryWindow>>();
    w.single_mut(app.world_mut())
        .expect("window")
        .set_cursor_position(Some(Vec2::new(200.0, 300.0)));

    let inv = app
        .world_mut()
        .spawn(Inventory {
            resources: [(ResourceId("ore".into()), 5)].into(),
            capacity: 0,
        })
        .id();

    let slot = app
        .world_mut()
        .spawn((
            InventorySlot { index: 0 },
            Node {
                width: Val::Px(48.0),
                height: Val::Px(48.0),
                ..default()
            },
            Transform::from_xyz(200.0, 300.0, 0.0),
            GlobalTransform::from_translation(Vec3::new(200.0, 300.0, 0.0)),
            Interaction::Pressed,
        ))
        .id();

    let grid = app
        .world_mut()
        .spawn(InventoryGrid {
            cols: 5,
            rows: 1,
            owner: inv,
        })
        .id();
    app.world_mut().entity_mut(grid).add_child(slot);
    app.update();

    let drag = app.world().resource::<DragState>();
    assert!(drag.active, "drag_start should activate on pressed slot");
    assert_eq!(drag.source_owner, Some(inv));
}

#[test]
fn test_drag_end_transfer_works() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<DragState>();
    app.insert_resource(ButtonInput::<MouseButton>::default());
    app.insert_resource(siege_factory::core::toast::ToastQueue(Vec::new()));
    app.add_systems(Update, siege_factory::economy::ui::drag_end);

    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let mut w = app
        .world_mut()
        .query_filtered::<&mut Window, With<PrimaryWindow>>();
    w.single_mut(app.world_mut())
        .expect("window")
        .set_cursor_position(Some(Vec2::new(200.0, 300.0)));

    let src = app
        .world_mut()
        .spawn(Inventory {
            resources: [(ResourceId("iron_ore".into()), 5)].into(),
            capacity: 0,
        })
        .id();
    let dst = app.world_mut().spawn(Inventory::new()).id();

    // Target slot at cursor position
    let slot = app
        .world_mut()
        .spawn((
            InventorySlot { index: 0 },
            Transform::from_xyz(200.0, 300.0, 0.0),
            GlobalTransform::from_translation(Vec3::new(200.0, 300.0, 0.0)),
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

    // Set drag state
    let mut d = app.world_mut().resource_mut::<DragState>();
    d.active = true;
    d.source_owner = Some(src);
    d.resource = Some(ResourceId("iron_ore".into()));
    d.amount = 1;

    // Press & release on the same frame
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
