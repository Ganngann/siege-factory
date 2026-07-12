use bevy::ecs::hierarchy::ChildOf;
use bevy::prelude::*;

use crate::core::utils::silent_despawn;
use crate::economy::components::{CloseButton, DragHandle};
use crate::economy::ui_components::ManagedByPanel;

// ── Components ──

#[derive(Component)]
pub struct WindowRoot;

// ── Drag resource ──

#[derive(Resource, Default)]
pub struct WindowDrag {
    pub dragging: bool,
    pub window_entity: Option<Entity>,
    pub cursor_start: Vec2,
    pub window_start_left: f32,
    pub window_start_top: f32,
}

// ── Generic window drag system ──

pub fn drag_window_system(
    mut drag: ResMut<WindowDrag>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut window_query: Query<(Entity, &mut Node), With<WindowRoot>>,
    handles: Query<(&Interaction, &ChildOf), (Changed<Interaction>, With<DragHandle>)>,
) {
    if window_query.is_empty() {
        *drag = WindowDrag::default();
        return;
    }

    let Ok(w) = windows.single() else { return };
    let Some(cursor) = w.cursor_position() else { return };

    if drag.dragging {
        if buttons.just_released(MouseButton::Left) {
            drag.dragging = false;
            drag.window_entity = None;
        } else if let Some(entity) = drag.window_entity
            && let Ok((_, mut node)) = window_query.get_mut(entity)
        {
            node.left = Val::Px(drag.window_start_left + cursor.x - drag.cursor_start.x);
            node.top = Val::Px(drag.window_start_top + cursor.y - drag.cursor_start.y);
        }
        return;
    }

    for (interaction, child_of) in handles.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let window_entity = child_of.0;
        if let Ok((_, node)) = window_query.get(window_entity) {
            let left = match node.left {
                Val::Px(v) => v,
                _ => continue,
            };
            let top = match node.top {
                Val::Px(v) => v,
                _ => continue,
            };
            drag.dragging = true;
            drag.window_entity = Some(window_entity);
            drag.cursor_start = cursor;
            drag.window_start_left = left;
            drag.window_start_top = top;
        }
    }
}

// ── Generic close button system ──

pub fn close_window_system(
    mut commands: Commands,
    buttons: Query<(Entity, &Interaction), (Changed<Interaction>, With<CloseButton>)>,
    parents: Query<&ChildOf>,
    windows: Query<Entity, With<WindowRoot>>,
    managed: Query<&ManagedByPanel>,
) {
    for (entity, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let mut current = entity;
        loop {
            if windows.contains(current) {
                if managed.contains(current) {
                    break;
                }
                silent_despawn(&mut commands, current);
                break;
            }
            match parents.get(current) {
                Ok(child_of) => current = child_of.0,
                Err(_) => break,
            }
        }
    }
}
