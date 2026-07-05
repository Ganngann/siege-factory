use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::core::game_state::GameState;
use crate::core::input::{InputBinding, KeyBindings};
use crate::core::settings::Settings;
use crate::core::utils::silent_despawn;
use crate::economy::components::PeacefulMode;
use crate::save_load::{SaveManager, save_path};

// ── TOML types ──

#[derive(Deserialize)]
struct MenuToml {
    screen: HashMap<String, TomlScreen>,
}

#[derive(Deserialize)]
struct TomlScreen {
    title: String,
    subtitle: Option<String>,
    #[serde(default)]
    items: Vec<TomlItem>,
}

#[derive(Deserialize)]
struct TomlItem {
    id: String,
    label: String,
    action: String,
    #[serde(default)]
    target: Option<String>,
}

// ── Runtime types ──

#[derive(Debug, Clone, Resource)]
pub struct MainMenuDef {
    pub screens: HashMap<String, ScreenDef>,
}

#[derive(Debug, Clone)]
pub struct ScreenDef {
    pub title: String,
    pub subtitle: Option<String>,
    pub items: Vec<MenuItemDef>,
}

#[derive(Debug, Clone)]
pub struct MenuItemDef {
    pub id: String,
    pub label: String,
    pub action: MenuAction,
}

#[derive(Debug, Clone)]
pub enum MenuAction {
    StartGame,
    StartPeaceful,
    OpenScreen(String),
    Back,
    Quit,
    Rebind(String),
    LoadGame,
}

#[derive(Debug, Resource)]
pub struct MenuNav {
    pub stack: Vec<String>,
    pub selection: usize,
}

impl Default for MenuNav {
    fn default() -> Self {
        Self {
            stack: vec!["main_menu".to_string()],
            selection: 0,
        }
    }
}

#[derive(Debug, Default, Resource)]
pub struct RebindState(pub Option<String>);

// ── UI Components ──

#[derive(Component)]
pub struct MenuRoot;

#[derive(Component)]
pub struct MenuCamera;

#[derive(Component)]
pub struct MenuItemComp(pub String, pub MenuAction);

#[derive(Component)]
pub struct MenuIndex(pub usize);

#[derive(Component)]
pub struct MenuRebindPrompt;

// ── Load ──

impl MainMenuDef {
    pub fn load() -> Self {
        let raw: MenuToml = toml::from_str(include_str!("../../data/main_menu.toml"))
            .expect("failed to parse data/main_menu.toml");

        let screens = raw
            .screen
            .into_iter()
            .map(|(id, ts)| {
                let items = ts
                    .items
                    .into_iter()
                    .filter_map(|ti| {
                        let action = match ti.action.as_str() {
                            "StartGame" => MenuAction::StartGame,
                            "StartPeaceful" => MenuAction::StartPeaceful,
                            "OpenScreen" => MenuAction::OpenScreen(
                                ti.target.clone().unwrap_or_default(),
                            ),
                            "Back" => MenuAction::Back,
                            "Quit" => MenuAction::Quit,
                            "LoadGame" => MenuAction::LoadGame,
                            _ => return None,
                        };
                        Some(MenuItemDef {
                            id: ti.id,
                            label: ti.label,
                            action,
                        })
                    })
                    .collect();
                (
                    id,
                    ScreenDef {
                        title: ts.title,
                        subtitle: ts.subtitle,
                        items,
                    },
                )
            })
            .collect();

        Self { screens }
    }
}

// ── Helpers ──

fn build_rebind_items(bindings: &KeyBindings) -> Vec<(String, String, MenuAction)> {
    let mut items: Vec<_> = bindings
        .all()
        .into_iter()
        .map(|(action, binding)| {
            let label = format!("{}  :  {}", action, binding_to_str(binding));
            (format!("rebind_{}", action), label, MenuAction::Rebind(action))
        })
        .collect();
    items.push(("back".to_string(), "Back".to_string(), MenuAction::Back));
    items
}

fn binding_to_str(binding: InputBinding) -> String {
    match binding {
        InputBinding::Key(k) => format!("{:?}", k),
        InputBinding::Mouse(m) => format!("{:?}", m),
    }
}

fn spawn_current_screen(
    commands: &mut Commands,
    def: &MainMenuDef,
    nav: &MenuNav,
    bindings: &KeyBindings,
    camera_exists: bool,
) {
    let screen_id = nav.stack.last().cloned().unwrap_or_default();
    let Some(screen) = def.screens.get(&screen_id) else { return };

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
                parent.spawn((
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

// ── Systems ──

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

/// Handles menu navigation and UI rebuild when nav state changes.
pub fn menu_navigation(
    mut commands: Commands,
    mut nav: ResMut<MenuNav>,
    mut rebind: ResMut<RebindState>,
    def: Res<MainMenuDef>,
    keys: Res<ButtonInput<KeyCode>>,
    _mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
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
    // Skip navigation while in rebind mode
    if rebind.0.is_some() {
        return;
    }

    // Rebuild UI if nav stack changed (ignore selection changes from hover)
    if *last_nav != nav.stack {
        for entity in &root_query {
            silent_despawn(&mut commands, entity);
        }
        spawn_current_screen(&mut commands, &def, &nav, &bindings, !camera_query.is_empty());
        *last_nav = nav.stack.clone();
        return;
    }

    // Get current screen items list
    let screen_id = nav.stack.last().cloned().unwrap_or_default();
    let Some(screen) = def.screens.get(&screen_id) else { return };

    let items: Vec<MenuItemAction> = if screen_id == "keybindings" {
        build_rebind_items(&bindings)
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

    // ── Mouse hover → update selection ──
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

        // Sync text colour
        let target = if index.0 == nav.selection
            || *interaction == Interaction::Hovered
            || *interaction == Interaction::Pressed
        {
            Color::srgb(1.0, 1.0, 1.0)
        } else {
            Color::srgb(0.6, 0.6, 0.7)
        };
        if let Some(child) = children.first() {
            if let Ok(mut tc) = text_colors.get_mut(*child) {
                tc.0 = target;
            }
        }
    }

    // ── Keyboard navigation ──
    if keys.just_pressed(KeyCode::ArrowUp) {
        nav.selection = nav.selection.saturating_sub(1);
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        nav.selection = (nav.selection + 1).min(max_idx);
    }

    // Escape = Back (if not on main_menu)
    if keys.just_pressed(KeyCode::Escape) {
        if nav.stack.len() > 1 {
            nav.selection = 0;
            nav.stack.pop();
        }
        return;
    }

    // ── Activate item ──
    let activate_idx = mouse_pressed
        .or_else(|| {
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
            MenuAction::Rebind(action) => {
                rebind.0 = Some(action.clone());
            }
            MenuAction::Quit => {
                // Disabled — requires bevy AppExit feature
            }
        }
    }
}

// ── Internal helpers ──

struct MenuItemAction {
    action: MenuAction,
}



/// Handles key capture during rebind mode.
pub fn menu_rebind_handler(
    mut commands: Commands,
    mut rebind: ResMut<RebindState>,
    mut bindings: ResMut<KeyBindings>,
    mut settings: ResMut<Settings>,
    mut nav: ResMut<MenuNav>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    prompt_query: Query<Entity, With<MenuRebindPrompt>>,
) {
    let Some(ref action) = rebind.0.clone() else {
        // Clean up prompt if rebind was somehow cancelled externally
        for entity in &prompt_query {
            commands.entity(entity).despawn();
        }
        return;
    };

    // Spawn prompt overlay if not already present
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
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
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
                    TextFont::from_font_size(28.0),
                    TextColor(Color::srgb(1.0, 1.0, 0.8)),
                ));
            });
    }

    // Escape cancels rebind mode
    if keys.just_pressed(KeyCode::Escape) {
        for entity in &prompt_query {
            commands.entity(entity).despawn();
        }
        rebind.0 = None;
        return;
    }

    // Try to capture a key (skip common navigation keys)
    for key in keys.get_just_pressed() {
        if matches!(key, KeyCode::Enter | KeyCode::ArrowUp | KeyCode::ArrowDown | KeyCode::Escape) {
            continue;
        }
        let binding = InputBinding::Key(*key);
        bindings.set(action, binding);
        settings
            .keybindings
            .insert(action.clone(), format!("{:?}", key));
        settings.save();
        for entity in &prompt_query {
            commands.entity(entity).despawn();
        }
        rebind.0 = None;
        // Pop back to parent screen so user sees the change
        if nav.stack.len() > 1 {
            nav.selection = 0;
            nav.stack.pop();
        }
        return;
    }

    // Try to capture a mouse button
    for btn in mouse.get_just_pressed() {
        // Skip mouse clicks that might be on the menu
        // Only capture MouseMiddle to avoid interfering with menu clicks
        if *btn == MouseButton::Middle {
            let binding = InputBinding::Mouse(*btn);
            bindings.set(action, binding);
            settings
                .keybindings
                .insert(action.clone(), format!("{:?}", btn));
            settings.save();
            for entity in &prompt_query {
                commands.entity(entity).despawn();
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
