use crate::economy::components::{
    DragState, DraggedItemVisual, InventoryGrid, InventorySlot, Player,
};
use crate::economy::resource::{Inventory, ResourceRegistry};
use crate::economy::window::{BG_SECTION, spawn_window};
use bevy::prelude::*;
use bevy::ui::widget::ImageNode;
use bevy::ui::UiTransform;

const SLOT_SIZE: f32 = 48.0;
const SLOT_GAP: f32 = 4.0;
const GRID_COLS: usize = 5;

#[derive(Component)]
pub struct InventoryPanel;

pub fn toggle_inventory_panel(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    panel_query: Query<Entity, With<InventoryPanel>>,
    player_query: Query<Entity, With<Player>>,
) {
    if !keys.just_pressed(KeyCode::KeyI) {
        return;
    }

    if let Ok(entity) = panel_query.single() {
        commands.entity(entity).despawn();
        return;
    }

    let Ok(player_entity) = player_query.single() else {
        return;
    };

    let cols = GRID_COLS;
    let rows = 4;
    let w = 280.0;
    let h = rows as f32 * (SLOT_SIZE + SLOT_GAP) + SLOT_GAP + 50.0;

    let panel_root = spawn_window(
        &mut commands,
        "Inventaire",
        w,
        h,
        100.0,
        80.0,
        None,
        |parent| {
            parent
                .spawn((
                    InventoryGrid {
                        cols,
                        rows,
                        owner: player_entity,
                    },
                    Transform::default(),
                    UiTransform::default(),
                    Node {
                        width: Val::Px(cols as f32 * (SLOT_SIZE + SLOT_GAP) + SLOT_GAP * 2.0),
                        padding: UiRect::all(Val::Px(SLOT_GAP)),
                        display: Display::Flex,
                        flex_wrap: FlexWrap::Wrap,
                        align_content: AlignContent::FlexStart,
                        margin: UiRect::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(BG_SECTION),
                ))
                .with_children(|grid| {
                    for i in 0..rows * cols {
                        grid.spawn((
                            InventorySlot { index: i },
                            Button,
                            Transform::default(),
                            UiTransform::default(),
                            Node {
                                width: Val::Px(SLOT_SIZE),
                                height: Val::Px(SLOT_SIZE),
                                flex_shrink: 0.0,
                                margin: UiRect::axes(
                                    Val::Px(SLOT_GAP / 2.0),
                                    Val::Px(SLOT_GAP / 2.0),
                                ),
                                border: UiRect::all(Val::Px(1.0)),
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BorderColor::all(Color::srgba(0.3, 0.3, 0.4, 1.0)),
                            BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 1.0)),
                            ImageNode::default(),
                            Text::new(String::new()),
                            TextFont::from_font_size(11.0),
                            TextColor(Color::WHITE),
                        ));
                    }
                });
        },
    );
    commands.entity(panel_root).insert(InventoryPanel);
}

// ── Update slot visuals from inventory data ──

pub fn update_inventory_grids(
    inv_grid_query: Query<(&InventoryGrid, &Children)>,
    mut slot_query: Query<(
        &mut BackgroundColor,
        &mut BorderColor,
        &InventorySlot,
        Option<&mut ImageNode>,
        Option<&mut Text>,
    )>,
    inv_query: Query<&Inventory>,
    registry: Res<ResourceRegistry>,
    textures: Res<crate::rendering::TextureCache>,
) {
    for (grid, children) in inv_grid_query.iter() {
        let Ok(inv) = inv_query.get(grid.owner) else {
            continue;
        };

        for child in children.iter() {
            if let Ok((mut bg, mut border, slot, image, text)) = slot_query.get_mut(child) {
                if let Some((rid, amount)) = inv.slot_content(slot.index) {
                    bg.0 = registry
                        .get_opt(&rid.0)
                        .map(|d| d.color)
                        .unwrap_or(Color::srgba(0.3, 0.3, 0.4, 1.0));
                    border.top = Color::srgba(0.5, 0.5, 0.6, 1.0);
                    border.bottom = Color::srgba(0.5, 0.5, 0.6, 1.0);
                    border.left = Color::srgba(0.5, 0.5, 0.6, 1.0);
                    border.right = Color::srgba(0.5, 0.5, 0.6, 1.0);
                    if let Some(mut img) = image {
                        img.image = textures.base(&rid.0);
                    }
                    if let Some(mut t) = text {
                        t.0 = format!("{}", amount);
                    }
                } else {
                    bg.0 = Color::srgba(0.08, 0.08, 0.12, 1.0);
                    if let Some(mut img) = image {
                        img.image = Handle::default();
                    }
                    if let Some(mut t) = text {
                        t.0 = String::new();
                    }
                }
            }
        }
    }
}

pub fn cleanup_inventory_panel(
    mut commands: Commands,
    panel_query: Query<Entity, With<InventoryPanel>>,
) {
    for entity in panel_query.iter() {
        commands.entity(entity).despawn();
    }
}

// ── Drag & Drop (rect hit-test, no Interaction dependency) ──

pub fn drag_start(
    mut drag: ResMut<DragState>,
    windows: Query<&Window>,
    keys: Res<ButtonInput<KeyCode>>,
    slots: Query<
        (
            Entity,
            &InventorySlot,
            &Interaction,
        ),
        (With<InventorySlot>, Changed<Interaction>),
    >,
    grids: Query<(&InventoryGrid, &Children)>,
    inv_query: Query<&Inventory>,
    mut commands: Commands,
) {
    if drag.active {
        return;
    }

    let cursor = windows
        .iter()
        .next()
        .and_then(|w| w.cursor_position());

    for (slot_entity, slot, interaction) in slots.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        for (grid, children) in grids.iter() {
            if !children.iter().any(|c| c == slot_entity) {
                continue;
            }
            let Ok(inv) = inv_query.get(grid.owner) else {
                continue;
            };
            if let Some((rid, slot_amount)) = inv.slot_content(slot.index) {
                let slot_amount = *slot_amount;
                let amount = if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
                    slot_amount.div_ceil(2)
                } else if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
                    1
                } else {
                    slot_amount
                };

                drag.active = true;
                drag.source_owner = Some(grid.owner);
                drag.source_slot_index = slot.index;
                drag.resource = Some(rid.clone());
                drag.amount = amount;

                let visual = commands
                    .spawn((
                        DraggedItemVisual,
                        Text::new(format!("{} ×{}", rid.display_name(), amount)),
                        TextFont::from_font_size(14.0),
                        TextColor(Color::WHITE),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(cursor.map_or(0.0, |c| c.x - 20.0)),
                            top: Val::Px(cursor.map_or(0.0, |c| c.y - 10.0)),
                            padding: UiRect::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.9)),
                        ZIndex(1000),
                    ))
                    .id();
                drag.visual = Some(visual);
                return;
            }
        }
    }
}

pub fn drag_update(
    drag: Res<DragState>,
    windows: Query<&Window>,
    mut visual_query: Query<&mut Node, (With<DraggedItemVisual>, Without<InventorySlot>)>,
) {
    if !drag.active {
        return;
    }
    let Some(visual) = drag.visual else { return };
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    if let Ok(mut node) = visual_query.get_mut(visual) {
        node.left = Val::Px(cursor.x - 20.0);
        node.top = Val::Px(cursor.y - 10.0);
    }
}

pub fn drag_end(
    mut drag: ResMut<DragState>,
    buttons: Res<ButtonInput<MouseButton>>,
    slots: Query<(Entity, &InventorySlot, &Interaction), (With<InventorySlot>, Without<InventoryGrid>)>,
    grids: Query<(&InventoryGrid, &Children), Without<InventorySlot>>,
    mut inv_query: Query<&mut Inventory>,
    mut commands: Commands,
    mut toast_queue: ResMut<crate::core::toast::ToastQueue>,
) {
    if !drag.active {
        return;
    }
    if !buttons.just_released(MouseButton::Left) {
        return;
    }

    if let Some(visual) = drag.visual {
        commands.entity(visual).despawn();
    }

    let src_owner = drag.source_owner;
    let src_idx = drag.source_slot_index;
    let resource = drag.resource.clone();
    let amount = drag.amount;
    drag.reset();

    let Some(ref resource) = resource else { return };
    let Some(src_owner) = src_owner else { return };

    // Find target slot via Interaction::Hovered (set by bevy's ui_focus_system)
    let mut dst_owner: Option<Entity> = None;
    let mut dst_idx: Option<usize> = None;
    'outer: for (slot_entity, slot, interaction) in slots.iter() {
        if *interaction != Interaction::Hovered {
            continue;
        }
        for (grid, children) in grids.iter() {
            if children.iter().any(|c| c == slot_entity) {
                dst_owner = Some(grid.owner);
                dst_idx = Some(slot.index);
                break 'outer;
            }
        }
    }

    let Some(dst_owner) = dst_owner else { return };
    let Some(dst_idx) = dst_idx else { return };

    // ── Same inventory ──
    if dst_owner == src_owner {
        if let Ok(mut inv) = inv_query.get_mut(src_owner) {
            if src_idx == dst_idx {
                return;
            }
            let src_amount = inv.slot_content(src_idx).map(|(_, a)| *a).unwrap_or(0);
            if src_amount == 0 {
                return;
            }
            let dst_empty = inv.slot_content(dst_idx).is_none();
            let dst_same = inv
                .slot_content(dst_idx)
                .map(|(r, _)| *r == *resource)
                .unwrap_or(false);

            if amount >= src_amount && !dst_same {
                // Full stack to different/empty → swap
                inv.swap_slots(src_idx, dst_idx);
            } else if dst_empty || dst_same {
                // Merge (full or partial) into same resource or empty slot
                let max = src_idx.max(dst_idx);
                if max >= inv.slots.len() {
                    inv.slots.resize(max + 1, None);
                }
                let slot_a;
                let slot_b;
                if src_idx < dst_idx {
                    let (left, right) = inv.slots.split_at_mut(dst_idx);
                    slot_a = &mut left[src_idx];
                    slot_b = &mut right[0];
                } else {
                    let (left, right) = inv.slots.split_at_mut(src_idx);
                    slot_b = &mut left[dst_idx];
                    slot_a = &mut right[0];
                }
                let src = slot_a.take();
                if let Some((res, amt)) = src {
                    let remaining = amt.saturating_sub(amount);
                    if remaining > 0 {
                        *slot_a = Some((res.clone(), remaining));
                    }
                    match slot_b {
                        Some((_, da)) => *da += if amount >= amt { amt } else { amount },
                        None => *slot_b = Some((res, if amount >= amt { amt } else { amount })),
                    }
                }
            } else {
                // Different resource → swap entire slots
                inv.swap_slots(src_idx, dst_idx);
            }
        }
        return;
    }

    // ── Cross-inventory transfer (1 unit) ──
    let removed = {
        if let Ok(mut inv) = inv_query.get_mut(src_owner) {
            if inv.get(resource) >= amount {
                inv.remove(resource, amount);
                true
            } else {
                false
            }
        } else {
            false
        }
    };

    if !removed {
        return;
    }

    if let Ok(mut inv) = inv_query.get_mut(dst_owner) {
        if inv.capacity == 0 || !inv.is_full() {
            inv.add(resource, amount);
            toast_queue
                .0
                .push(format!("Transféré {} ×{}", resource.display_name(), amount));
        } else {
            if let Ok(mut src_inv) = inv_query.get_mut(src_owner) {
                src_inv.add(resource, amount);
            }
            toast_queue.0.push("Destination pleine".to_string());
        }
    }
}
