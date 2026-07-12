use bevy::prelude::*;

use crate::core::game_font::tf;

use super::types::*;
use crate::core::input::{InputBinding, KeyBindings};
use crate::core::modding::ModRegistry;
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

pub(crate) fn build_mod_items(registry: &ModRegistry) -> Vec<(String, String, MenuAction)> {
    let mut items: Vec<_> = registry
        .mods
        .iter()
        .filter(|am| am.manifest.id != "base")
        .map(|am| {
            let check = if am.enabled { "[x]" } else { "[ ]" };
            let label = format!("{}  {}  v{}", check, am.manifest.name, am.manifest.version);
            (
                am.manifest.id.clone(),
                label,
                MenuAction::ToggleMod(am.manifest.id.clone()),
            )
        })
        .collect();
    if items.is_empty() {
        items.push((
            "no_extra_mods".to_string(),
            "(No additional mods found)".to_string(),
            MenuAction::Back,
        ));
    }
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
    registry: &ModRegistry,
    camera_exists: bool,
) {
    let screen_id = nav.stack.last().cloned().unwrap_or_default();
    let Some(screen) = def.screens.get(&screen_id) else {
        return;
    };
    let cfg = &def.config;

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
            BackgroundColor(cfg.bg_color),
        ))
        .with_children(|parent| {
            parent.spawn((
                MenuRoot,
                Text::new(&screen.title),
                tf(cfg.title_font_size),
                TextColor(cfg.title_color),
            ));

            if let Some(sub) = &screen.subtitle {
                parent.spawn((
                    MenuRoot,
                    Text::new(sub.as_str()),
                    tf(cfg.subtitle_font_size),
                    TextColor(cfg.subtitle_color),
                ));
            }

            parent.spawn((
                MenuRoot,
                Text::new(""),
                TextFont::default(),
                TextColor(Color::WHITE),
            ));

            let is_keybindings = screen_id == "keybindings";
            let is_mods = screen_id == "mods";
            let items: Vec<(String, String, MenuAction)> = if is_keybindings {
                build_rebind_items(bindings)
            } else if is_mods {
                build_mod_items(registry)
            } else {
                screen
                    .items
                    .iter()
                    .map(|it| (it.id.clone(), it.label.clone(), it.action.clone()))
                    .collect()
            };

            for (idx, (id, label, action)) in items.iter().enumerate() {
                let color = if idx == nav.selection {
                    cfg.item_selected_color
                } else {
                    cfg.item_default_color
                };
                parent
                    .spawn((
                        MenuRoot,
                        MenuItemComp(id.clone(), action.clone()),
                        MenuIndex(idx),
                        Button,
                        Interaction::default(),
                        Node {
                            padding: UiRect::axes(Val::Px(cfg.item_padding_x), Val::Px(cfg.item_padding_y)),
                            min_width: Val::Px(cfg.item_min_width),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|p| {
                        p.spawn((
                            MenuRoot,
                            Text::new(label.as_str()),
                            tf(cfg.item_font_size),
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
