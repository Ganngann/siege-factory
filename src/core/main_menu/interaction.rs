use bevy::prelude::*;

use crate::core::game_font::tf;

use super::spawn::{build_mod_items, build_rebind_items, spawn_current_screen};
use super::types::*;
use crate::core::game_state::GameState;
use crate::core::input::{InputBinding, KeyBindings};
use crate::core::modding::ModRegistry;
use crate::core::settings::Settings;
use crate::core::utils::silent_despawn;
use crate::economy::components::PeacefulMode;
use crate::save_load::{SaveManager, save_path};

struct MenuItemAction {
    action: MenuAction,
}

pub fn menu_navigation(
    mut commands: Commands,
    mut nav: ResMut<MenuNav>,
    mut rebind: ResMut<RebindState>,
    def: Res<MainMenuDef>,
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    mut registry: ResMut<ModRegistry>,
    mut next_state: ResMut<NextState<GameState>>,
    mut fresh_game: ResMut<crate::core::game_state::IsFreshGame>,
    mut peaceful: ResMut<PeacefulMode>,
    mut save_mgr: ResMut<SaveManager>,
    root_query: Query<Entity, With<MenuRoot>>,
    buttons: Query<(&Interaction, &MenuIndex, &MenuItemComp, &Children)>,
    mut text_colors: Query<&mut TextColor>,
    camera_query: Query<Entity, With<MenuCamera>>,
    mut last_nav: Local<Vec<String>>,
) {
    if rebind.0.is_some() {
        return;
    }

    if *last_nav != nav.stack {
        for entity in &root_query {
            silent_despawn(&mut commands, entity);
        }
        spawn_current_screen(
            &mut commands,
            &def,
            &nav,
            &bindings,
            &registry,
            !camera_query.is_empty(),
        );
        *last_nav = nav.stack.clone();
        return;
    }

    let screen_id = nav.stack.last().cloned().unwrap_or_default();
    let Some(screen) = def.screens.get(&screen_id) else {
        return;
    };

    let items: Vec<MenuItemAction> = if screen_id == "keybindings" {
        build_rebind_items(&bindings)
            .into_iter()
            .map(|(_id, _, action)| MenuItemAction { action })
            .collect()
    } else if screen_id == "mods" {
        build_mod_items(&registry)
            .into_iter()
            .map(|(_id, _, action)| MenuItemAction { action })
            .collect()
    } else {
        screen
            .items
            .iter()
            .map(|it| MenuItemAction {
                action: it.action.clone(),
            })
            .collect()
    };

    if items.is_empty() {
        return;
    }

    let max_idx = items.len().saturating_sub(1);
    if nav.selection > max_idx {
        nav.selection = max_idx;
    }

    let mut mouse_pressed: Option<usize> = None;
    for (interaction, index, _comp, children) in buttons.iter() {
        match *interaction {
            Interaction::Hovered => {
                nav.selection = index.0;
            }
            Interaction::Pressed => {
                mouse_pressed = Some(index.0);
            }
            _ => {}
        }

        let target = if index.0 == nav.selection
            || *interaction == Interaction::Hovered
            || *interaction == Interaction::Pressed
        {
            def.config.item_selected_color
        } else {
            def.config.item_default_color
        };
        if let Some(child) = children.first()
            && let Ok(mut tc) = text_colors.get_mut(*child) {
                tc.0 = target;
            }
    }

    if keys.just_pressed(KeyCode::ArrowUp) {
        nav.selection = nav.selection.saturating_sub(1);
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        nav.selection = (nav.selection + 1).min(max_idx);
    }

    if keys.just_pressed(KeyCode::Escape) {
        if nav.stack.len() > 1 {
            nav.selection = 0;
            nav.stack.pop();
        }
        return;
    }

    let activate_idx = mouse_pressed.or_else(|| {
        if keys.just_pressed(KeyCode::Enter) {
            Some(nav.selection)
        } else {
            None
        }
    });

    if let Some(idx) = activate_idx {
        let Some(item) = items.get(idx) else { return };
        match &item.action {
            MenuAction::StartGame => {
                fresh_game.0 = true;
                peaceful.0 = false;
                next_state.set(GameState::Playing);
            }
            MenuAction::StartPeaceful => {
                fresh_game.0 = true;
                peaceful.0 = true;
                next_state.set(GameState::Playing);
            }
            MenuAction::LoadGame => {
                let path = save_path();
                if path.exists() {
                    save_mgr.load_requested = Some(path.to_string_lossy().to_string());
                    fresh_game.0 = false;
                    next_state.set(GameState::Loading);
                }
            }
            MenuAction::OpenScreen(target) => {
                if def.screens.contains_key(target.as_str()) {
                    nav.stack.push(target.clone());
                    nav.selection = 0;
                }
            }
            MenuAction::Back => {
                if nav.stack.len() > 1 {
                    nav.selection = 0;
                    nav.stack.pop();
                }
            }
            MenuAction::ToggleMod(id) => {
                registry.toggle(id);
                *last_nav = Vec::new();
            }
            MenuAction::Rebind(action) => {
                rebind.0 = Some(action.clone());
            }
            MenuAction::Quit => {
            }
        }
    }
}

pub fn menu_rebind_handler(
    mut commands: Commands,
    mut rebind: ResMut<RebindState>,
    mut bindings: ResMut<KeyBindings>,
    mut settings: ResMut<Settings>,
    mut nav: ResMut<MenuNav>,
    def: Res<MainMenuDef>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    prompt_query: Query<Entity, With<MenuRebindPrompt>>,
) {
    let Some(ref action) = rebind.0.clone() else {
        for entity in &prompt_query {
            silent_despawn(&mut commands, entity);
        }
        return;
    };

    if prompt_query.is_empty() {
        commands
            .spawn((
                MenuRoot,
                MenuRebindPrompt,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(def.config.rebind_bg_color),
                ZIndex(100),
            ))
            .with_children(|parent| {
                parent.spawn((
                    MenuRoot,
                    MenuRebindPrompt,
                    Text::new(format!(
                        "Press a key for \"{}\"...\n\n(ESC to cancel)",
                        action
                    )),
                    tf(def.config.rebind_font_size),
                    TextColor(def.config.rebind_text_color),
                ));
            });
    }

    if keys.just_pressed(KeyCode::Escape) {
        for entity in &prompt_query {
            silent_despawn(&mut commands, entity);
        }
        rebind.0 = None;
        return;
    }

    for key in keys.get_just_pressed() {
        if matches!(
            key,
            KeyCode::Enter | KeyCode::ArrowUp | KeyCode::ArrowDown | KeyCode::Escape
        ) {
            continue;
        }
        let binding = InputBinding::Key(*key);
        bindings.set(action, binding);
        settings
            .keybindings
            .insert(action.clone(), binding.to_string());
        settings.save();
        for entity in &prompt_query {
            silent_despawn(&mut commands, entity);
        }
        rebind.0 = None;
        if nav.stack.len() > 1 {
            nav.selection = 0;
            nav.stack.pop();
        }
        return;
    }

    for btn in mouse.get_just_pressed() {
        if *btn == MouseButton::Middle {
            let binding = InputBinding::Mouse(*btn);
            bindings.set(action, binding);
            settings
                .keybindings
                .insert(action.clone(), binding.to_string());
            settings.save();
            for entity in &prompt_query {
                silent_despawn(&mut commands, entity);
            }
            rebind.0 = None;
            if nav.stack.len() > 1 {
                nav.selection = 0;
                nav.stack.pop();
            }
            return;
        }
    }
}
