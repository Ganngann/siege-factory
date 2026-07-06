use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

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
