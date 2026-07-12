use crate::core::game_font::tf;
use crate::core::toast::ToastQueue;
use crate::core::utils::silent_despawn;
use crate::economy::components::{
    DragState, DraggedItemVisual, InventoryGrid, InventorySlot,
};
use crate::economy::resource::{Inventory, ResourceRegistry};
use crate::rendering::TextureCache;
use bevy::prelude::*;

#[derive(Component)]
pub struct InventoryPanel;

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
    textures: Res<TextureCache>,
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

// ── Drag & Drop ──

pub fn drag_start(
    mut drag: ResMut<DragState>,
    windows: Query<&Window>,
    keys: Res<ButtonInput<KeyCode>>,
    slots: Query<
        (Entity, &InventorySlot, &Interaction),
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
                        tf(14.0),
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
    mut toast_queue: ResMut<ToastQueue>,
) {
    if !drag.active {
        return;
    }
    if !buttons.just_released(MouseButton::Left) {
        return;
    }

    if let Some(visual) = drag.visual {
        silent_despawn(&mut commands, visual);
    }

    let src_owner = drag.source_owner;
    let src_idx = drag.source_slot_index;
    let resource = drag.resource.clone();
    let amount = drag.amount;
    drag.reset();

    let Some(ref resource) = resource else { return };
    let Some(src_owner) = src_owner else { return };

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
                inv.swap_slots(src_idx, dst_idx);
            } else if dst_empty || dst_same {
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
                inv.swap_slots(src_idx, dst_idx);
            }
        }
        return;
    }

    // ── Cross-inventory transfer ──
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
