use bevy::prelude::*;

use crate::core::game_font::tf;
use crate::core::game_state::GameState;
use crate::core::modding::ModRegistry;
use crate::core::utils::silent_despawn;
use crate::save_load::{SaveManager, SaveRequested, save_path};

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct SaveButton;

#[derive(Component)]
pub struct LoadButton;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct QuitButton;

#[derive(Resource)]
pub struct PauseMenuConfig {
    pub title: String,
    pub title_font_size: f32,
    pub title_color: Color,
    pub overlay_opacity: f32,
    pub panel_bg: Color,
    pub panel_outline: Color,
    pub button_width: f32,
    pub button_height: f32,
    pub button_bg: Color,
    pub button_text_color: Color,
    pub button_font_size: f32,
    pub padding: f32,
    pub gap: f32,
}

fn parse_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::srgb(0.5, 0.5, 0.5);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
    Color::srgb(r, g, b)
}

impl PauseMenuConfig {
    pub fn load(mods: &ModRegistry) -> Self {
        let content = mods.load_data("panel_pause_menu.toml").unwrap_or_default();
        let Ok(config) = toml::from_str::<toml::Value>(&content) else {
            return Self::default();
        };
        Self {
            title: config
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("PAUSED")
                .to_string(),
            title_font_size: config
                .get("title_font_size")
                .and_then(|v| v.as_float())
                .unwrap_or(28.0) as f32,
            title_color: config
                .get("title_color")
                .and_then(|v| v.as_str())
                .map(parse_hex)
                .unwrap_or(Color::srgb(0.8, 0.8, 1.0)),
            overlay_opacity: config
                .get("overlay_opacity")
                .and_then(|v| v.as_float())
                .unwrap_or(0.6) as f32,
            panel_bg: config
                .get("panel_bg")
                .and_then(|v| v.as_str())
                .map(parse_hex)
                .unwrap_or(Color::srgb(0.1, 0.1, 0.15)),
            panel_outline: config
                .get("panel_outline")
                .and_then(|v| v.as_str())
                .map(parse_hex)
                .unwrap_or(Color::srgb(0.4, 0.4, 0.5)),
            button_width: config
                .get("button_width")
                .and_then(|v| v.as_float())
                .unwrap_or(200.0) as f32,
            button_height: config
                .get("button_height")
                .and_then(|v| v.as_float())
                .unwrap_or(40.0) as f32,
            button_bg: config
                .get("button_bg")
                .and_then(|v| v.as_str())
                .map(parse_hex)
                .unwrap_or(Color::srgb(0.2, 0.2, 0.3)),
            button_text_color: config
                .get("button_text_color")
                .and_then(|v| v.as_str())
                .map(parse_hex)
                .unwrap_or(Color::srgb(1.0, 1.0, 1.0)),
            button_font_size: config
                .get("button_font_size")
                .and_then(|v| v.as_float())
                .unwrap_or(16.0) as f32,
            padding: config
                .get("padding")
                .and_then(|v| v.as_float())
                .unwrap_or(24.0) as f32,
            gap: config.get("gap").and_then(|v| v.as_float()).unwrap_or(8.0) as f32,
        }
    }
}

impl Default for PauseMenuConfig {
    fn default() -> Self {
        Self {
            title: "PAUSED".into(),
            title_font_size: 28.0,
            title_color: Color::srgb(0.8, 0.8, 1.0),
            overlay_opacity: 0.6,
            panel_bg: Color::srgb(0.1, 0.1, 0.15),
            panel_outline: Color::srgb(0.4, 0.4, 0.5),
            button_width: 200.0,
            button_height: 40.0,
            button_bg: Color::srgb(0.2, 0.2, 0.3),
            button_text_color: Color::srgb(1.0, 1.0, 1.0),
            button_font_size: 16.0,
            padding: 24.0,
            gap: 8.0,
        }
    }
}

pub fn toggle_pause_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<crate::core::input::KeyBindings>,
    mut show: ResMut<crate::save_load::ShowPauseMenu>,
) {
    if bindings.just_pressed("cancel", &keys, &mouse) {
        show.0 = !show.0;
    }
}

pub fn spawn_pause_menu(
    mut commands: Commands,
    show: Res<crate::save_load::ShowPauseMenu>,
    config: Res<PauseMenuConfig>,
    panel_query: Query<Entity, With<PauseMenuRoot>>,
) {
    if show.0 && panel_query.is_empty() {
        let _ = commands
            .spawn((
                PauseMenuRoot,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, config.overlay_opacity)),
                Pickable::default(),
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(config.padding)),
                            row_gap: Val::Px(config.gap),
                            ..default()
                        },
                        BackgroundColor(config.panel_bg),
                        Outline {
                            width: Val::Px(2.0),
                            offset: Val::ZERO,
                            color: config.panel_outline,
                        },
                    ))
                    .with_children(|panel| {
                        panel.spawn((
                            Text::new(config.title.as_str()),
                            tf(config.title_font_size),
                            TextColor(config.title_color),
                            Node {
                                margin: UiRect::bottom(Val::Px(config.gap + 4.0)),
                                ..default()
                            },
                        ));
                        panel
                            .spawn((
                                SaveButton,
                                Button,
                                Node {
                                    width: Val::Px(config.button_width),
                                    height: Val::Px(config.button_height),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(config.button_bg),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Save Game"),
                                    tf(config.button_font_size),
                                    TextColor(config.button_text_color),
                                ));
                            });
                        panel
                            .spawn((
                                LoadButton,
                                Button,
                                Node {
                                    width: Val::Px(config.button_width),
                                    height: Val::Px(config.button_height),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(config.button_bg),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Load Game"),
                                    tf(config.button_font_size),
                                    TextColor(config.button_text_color),
                                ));
                            });
                        panel
                            .spawn((
                                ResumeButton,
                                Button,
                                Node {
                                    width: Val::Px(config.button_width),
                                    height: Val::Px(config.button_height),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(config.button_bg),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Resume"),
                                    tf(config.button_font_size),
                                    TextColor(config.button_text_color),
                                ));
                            });
                        panel
                            .spawn((
                                QuitButton,
                                Button,
                                Node {
                                    width: Val::Px(config.button_width),
                                    height: Val::Px(config.button_height),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(config.button_bg),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Main Menu"),
                                    tf(config.button_font_size),
                                    TextColor(config.button_text_color),
                                ));
                            });
                    });
            });
    } else if !show.0 {
        for entity in &panel_query {
            silent_despawn(&mut commands, entity);
        }
    }
}

pub fn resume_interaction(
    query: Query<&Interaction, (Changed<Interaction>, With<ResumeButton>)>,
    mut show: ResMut<crate::save_load::ShowPauseMenu>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            show.0 = false;
        }
    }
}

pub fn quit_interaction(
    query: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut show: ResMut<crate::save_load::ShowPauseMenu>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            show.0 = false;
            next_state.set(GameState::Menu);
        }
    }
}

pub fn save_interaction(
    query: Query<&Interaction, (Changed<Interaction>, With<SaveButton>)>,
    mut show: ResMut<crate::save_load::ShowPauseMenu>,
    mut save_req: ResMut<SaveRequested>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            show.0 = false;
            save_req.0 = true;
        }
    }
}

pub fn load_interaction(
    query: Query<&Interaction, (Changed<Interaction>, With<LoadButton>)>,
    mut save_mgr: ResMut<SaveManager>,
    mut next_state: ResMut<NextState<GameState>>,
    mut show: ResMut<crate::save_load::ShowPauseMenu>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            show.0 = false;
            save_mgr.load_requested = Some(save_path().to_string_lossy().to_string());
            next_state.set(GameState::Loading);
        }
    }
}

pub fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuRoot>>) {
    for e in &query {
        silent_despawn(&mut commands, e);
    }
}
