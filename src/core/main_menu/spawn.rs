use bevy::prelude::*;

use super::types::*;
use crate::core::input::{InputBinding, KeyBindings};
use crate::core::utils::silent_despawn;

pub(crate) fn build_rebind_items(bindings: &KeyBindings) -> Vec<(String, String, MenuAction)> {
    let mut items: Vec<_> = bindings
        .all()
        .into_iter()
        .map(|(action, binding)| {
            let label = format!("{}  :  {}", action, binding_to_str(binding));
            (
                format!("rebind_{}", action),
                label,
                MenuAction::Rebind(action),
            )
        })
        .collect();
    items.push(("back".to_string(), "Back".to_string(), MenuAction::Back));
    items
}

pub(crate) fn binding_to_str(binding: InputBinding) -> String {
    binding.to_string()
}

pub(crate) fn spawn_current_screen(
    commands: &mut Commands,
    def: &MainMenuDef,
    nav: &MenuNav,
    bindings: &KeyBindings,
    camera_exists: bool,
) {
    let screen_id = nav.stack.last().cloned().unwrap_or_default();
    let Some(screen) = def.screens.get(&screen_id) else {
        return;
    };

    if !camera_exists {
        commands.spawn((Camera2d, MenuCamera));
    }

    commands
        .spawn((
            MenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 1.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                MenuRoot,
                Text::new(&screen.title),
                TextFont::from_font_size(48.0),
                TextColor(Color::srgb(0.8, 0.8, 1.0)),
            ));

            if let Some(sub) = &screen.subtitle {
                parent.spawn((
                    MenuRoot,
                    Text::new(sub.as_str()),
                    TextFont::from_font_size(16.0),
                    TextColor(Color::srgb(0.6, 0.6, 0.8)),
                ));
            }

            parent.spawn((
                MenuRoot,
                Text::new(""),
                TextFont::default(),
                TextColor(Color::WHITE),
            ));

            let is_keybindings = screen_id == "keybindings";
            let items: Vec<(String, String, MenuAction)> = if is_keybindings {
                build_rebind_items(bindings)
            } else {
                screen
                    .items
                    .iter()
                    .map(|it| (it.id.clone(), it.label.clone(), it.action.clone()))
                    .collect()
            };

            for (idx, (id, label, action)) in items.iter().enumerate() {
                let color = if idx == nav.selection {
                    Color::srgb(1.0, 1.0, 1.0)
                } else {
                    Color::srgb(0.6, 0.6, 0.7)
                };
                parent
                    .spawn((
                        MenuRoot,
                        MenuItemComp(id.clone(), action.clone()),
                        MenuIndex(idx),
                        Button,
                        Interaction::default(),
                        Node {
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(4.0)),
                            min_width: Val::Px(300.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|p| {
                        p.spawn((
                            MenuRoot,
                            Text::new(label.as_str()),
                            TextFont::from_font_size(20.0),
                            TextColor(color),
                        ));
                    });
            }
        });
}

pub fn despawn_menu_ui(
    mut commands: Commands,
    query: Query<Entity, With<MenuRoot>>,
    camera_query: Query<Entity, With<MenuCamera>>,
) {
    for entity in &query {
        silent_despawn(&mut commands, entity);
    }
    for entity in &camera_query {
        silent_despawn(&mut commands, entity);
    }
}
