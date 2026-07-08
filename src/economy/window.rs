use crate::economy::components::{CloseButton, DragHandle};
use bevy::ecs::hierarchy::ChildOf;
use bevy::prelude::*;
// ── Shared UI constants ──

pub const BG_WINDOW: Color = Color::srgba(0.08, 0.08, 0.16, 0.97);
pub const BG_SECTION: Color = Color::srgb(0.10, 0.10, 0.18);
pub const ACCENT: Color = Color::srgb(0.30, 0.55, 1.00);
pub const TEXT_PRIMARY: Color = Color::srgb(0.90, 0.90, 1.00);
pub const TEXT_SECONDARY: Color = Color::srgb(0.60, 0.60, 0.75);
pub const TEXT_GREEN: Color = Color::srgb(0.40, 0.85, 0.40);
pub const TEXT_YELLOW: Color = Color::srgb(0.85, 0.85, 0.35);
pub const BTN_CLOSE: Color = Color::srgb(0.50, 0.12, 0.12);
pub const BTN_ACTIVE: Color = Color::srgb(0.15, 0.45, 0.15);
pub const BTN_INACTIVE: Color = Color::srgb(0.30, 0.15, 0.15);
pub const HP_GREEN: Color = Color::srgb(0.20, 0.65, 0.20);
pub const BAR_BG: Color = Color::srgb(0.15, 0.15, 0.22);
pub const SEPARATOR: Color = Color::srgb(0.20, 0.20, 0.30);

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

// ── Spawn a standardized window ──

pub fn spawn_window(
    commands: &mut Commands,
    title: &str,
    width: f32,
    height: f32,
    x: f32,
    y: f32,
    _on_close: Option<Entity>,
    content: impl FnOnce(&mut bevy::ecs::hierarchy::ChildSpawnerCommands),
) -> Entity {
    commands
        .spawn((
            WindowRoot,
            Transform::default(),
            bevy::ui::UiTransform::default(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x),
                top: Val::Px(y),
                flex_direction: FlexDirection::Column,
                width: Val::Px(width),
                height: Val::Px(height),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(BG_WINDOW),
            Outline {
                width: Val::Px(1.0),
                offset: Val::ZERO,
                color: Color::srgb(0.30, 0.30, 0.45),
            },
            ZIndex(101),
        ))
        .with_children(|parent| {
            // ── Header (draggable + title + close) ──
            parent
                .spawn((
                    DragHandle,
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(36.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::horizontal(Val::Px(12.0)),
                        border: UiRect::bottom(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(BG_SECTION),
                    BorderColor {
                        top: SEPARATOR,
                        bottom: SEPARATOR,
                        left: SEPARATOR,
                        right: SEPARATOR,
                    },
                ))
                .with_children(|header| {
                    header.spawn((
                        Text::new(title),
                        TextFont::from_font_size(16.0),
                        TextColor(TEXT_PRIMARY),
                    ));
                    header
                        .spawn((
                            CloseButton,
                            Button,
                            Node {
                                width: Val::Px(26.0),
                                height: Val::Px(26.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_CLOSE),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("X"),
                                TextFont::from_font_size(14.0),
                                TextColor(Color::WHITE),
                            ));
                        });
                });
            // ── Content area ──
            content(parent);
        })
        .id()
}

// ── Section helper (used inside content callback) ──

pub fn spawn_section(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    title: &str,
    content: impl FnOnce(&mut bevy::ecs::hierarchy::ChildSpawnerCommands),
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                margin: UiRect::bottom(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(BG_SECTION),
        ))
        .with_children(|sec| {
            sec.spawn((
                Text::new(title),
                TextFont::from_font_size(11.0),
                TextColor(ACCENT),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));
            content(sec);
        });
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
            && let Ok((_, mut node)) = window_query.get_mut(entity) {
                node.left = Val::Px(drag.window_start_left + cursor.x - drag.cursor_start.x);
                node.top = Val::Px(drag.window_start_top + cursor.y - drag.cursor_start.y);
            }
        return;
    }

    // Start drag when a DragHandle is pressed (Interaction set by ui_focus_system)
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
) {
    for (entity, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // Walk up parent chain to find WindowRoot
        let mut current = entity;
        loop {
            if windows.contains(current) {
                commands.entity(current).despawn();
                break;
            }
            match parents.get(current) {
                Ok(child_of) => current = child_of.0,
                Err(_) => break,
            }
        }
    }
}
