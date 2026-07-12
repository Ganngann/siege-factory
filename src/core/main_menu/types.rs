use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::core::modding::ModRegistry;

// ── TOML types ──

#[derive(Deserialize)]
pub(crate) struct MenuToml {
    pub(crate) screen: HashMap<String, TomlScreen>,
}

#[derive(Deserialize)]
pub(crate) struct TomlScreen {
    pub(crate) title: String,
    pub(crate) subtitle: Option<String>,
    #[serde(default)]
    pub(crate) items: Vec<TomlItem>,
}

#[derive(Deserialize)]
pub(crate) struct TomlItem {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) action: String,
    #[serde(default)]
    pub(crate) target: Option<String>,
}

// ── Runtime types ──

#[derive(Debug, Clone, Resource)]
pub struct MainMenuDef {
    pub screens: HashMap<String, ScreenDef>,
    pub config: MainMenuConfig,
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
    ToggleMod(String),
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

#[derive(Debug, Clone, Resource)]
pub struct MainMenuConfig {
    pub bg_color: Color,
    pub title_font_size: f32,
    pub title_color: Color,
    pub subtitle_font_size: f32,
    pub subtitle_color: Color,
    pub item_font_size: f32,
    pub item_selected_color: Color,
    pub item_default_color: Color,
    pub item_padding_y: f32,
    pub item_padding_x: f32,
    pub item_min_width: f32,
    pub rebind_bg_color: Color,
    pub rebind_text_color: Color,
    pub rebind_font_size: f32,
}

impl MainMenuConfig {
    pub fn load(mods: &ModRegistry) -> Self {
        let content = mods.load_data("panel_main_menu.toml").unwrap_or_default();
        let Ok(config) = toml::from_str::<toml::Value>(&content) else {
            return Self::default();
        };
        Self {
            bg_color: config.get("bg_color").and_then(|v| v.as_str()).map(crate::core::utils::parse_hex_color).unwrap_or(Color::srgb(0.05, 0.05, 0.1)),
            title_font_size: config.get("title_font_size").and_then(|v| v.as_float()).unwrap_or(48.0) as f32,
            title_color: config.get("title_color").and_then(|v| v.as_str()).map(crate::core::utils::parse_hex_color).unwrap_or(Color::srgb(0.8, 0.8, 1.0)),
            subtitle_font_size: config.get("subtitle_font_size").and_then(|v| v.as_float()).unwrap_or(16.0) as f32,
            subtitle_color: config.get("subtitle_color").and_then(|v| v.as_str()).map(crate::core::utils::parse_hex_color).unwrap_or(Color::srgb(0.6, 0.6, 0.8)),
            item_font_size: config.get("item_font_size").and_then(|v| v.as_float()).unwrap_or(20.0) as f32,
            item_selected_color: config.get("item_selected_color").and_then(|v| v.as_str()).map(crate::core::utils::parse_hex_color).unwrap_or(Color::srgb(1.0, 1.0, 1.0)),
            item_default_color: config.get("item_default_color").and_then(|v| v.as_str()).map(crate::core::utils::parse_hex_color).unwrap_or(Color::srgb(0.6, 0.6, 0.7)),
            item_padding_y: config.get("item_padding_y").and_then(|v| v.as_float()).unwrap_or(4.0) as f32,
            item_padding_x: config.get("item_padding_x").and_then(|v| v.as_float()).unwrap_or(20.0) as f32,
            item_min_width: config.get("item_min_width").and_then(|v| v.as_float()).unwrap_or(300.0) as f32,
            rebind_bg_color: {
                let c = config.get("rebind_bg_color").and_then(|v| v.as_str()).map(crate::core::utils::parse_hex_color).unwrap_or(Color::srgb(0.0, 0.0, 0.0));
                let opacity = config.get("rebind_opacity").and_then(|v| v.as_float()).unwrap_or(0.75) as f32;
                let srgba = c.to_srgba();
                Color::srgba(srgba.red, srgba.green, srgba.blue, opacity)
            },
            rebind_text_color: config.get("rebind_text_color").and_then(|v| v.as_str()).map(crate::core::utils::parse_hex_color).unwrap_or(Color::srgb(1.0, 1.0, 0.8)),
            rebind_font_size: config.get("rebind_font_size").and_then(|v| v.as_float()).unwrap_or(28.0) as f32,
        }
    }
}

impl Default for MainMenuConfig {
    fn default() -> Self {
        Self {
            bg_color: Color::srgb(0.05, 0.05, 0.1),
            title_font_size: 48.0,
            title_color: Color::srgb(0.8, 0.8, 1.0),
            subtitle_font_size: 16.0,
            subtitle_color: Color::srgb(0.6, 0.6, 0.8),
            item_font_size: 20.0,
            item_selected_color: Color::srgb(1.0, 1.0, 1.0),
            item_default_color: Color::srgb(0.6, 0.6, 0.7),
            item_padding_y: 4.0,
            item_padding_x: 20.0,
            item_min_width: 300.0,
            rebind_bg_color: Color::srgba(0.0, 0.0, 0.0, 0.75),
            rebind_text_color: Color::srgb(1.0, 1.0, 0.8),
            rebind_font_size: 28.0,
        }
    }
}
