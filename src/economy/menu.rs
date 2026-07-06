use crate::economy::building::BuildingRegistry;
use crate::economy::unit_config::UnitConfig;
use bevy::prelude::*;
use serde::Deserialize;

// ── Menu action (leaf) ──

#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    Build(String),
    Spawn(String),
    Delete,
}

// ── Menu entry (recursive tree) ──

#[derive(Debug, Clone)]
pub enum MenuEntry {
    Action {
        label: String,
        action: MenuAction,
    },
    SubMenu {
        label: String,
        items: Vec<MenuEntry>,
    },
}

// ── Flat item for current level (for UI rendering) ──

#[derive(Debug, Clone, PartialEq)]
pub struct FlatItem {
    pub label: String,
    pub kind: FlatItemKind,
    pub cost_str: String,
    pub color: Color,
    pub texture_stem: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlatItemKind {
    Action(MenuAction),
    SubMenu,
}

// ── Resources ──

#[derive(Debug, Clone, Resource)]
pub struct MenuDef {
    pub root: Vec<MenuEntry>,
}

#[derive(Debug, Default, Resource)]
pub struct MenuState {
    /// Stack of indices into each level. Empty = root level.
    pub stack: Vec<usize>,
    /// Scroll offset in the current level (how many items scrolled past).
    pub scroll: usize,
}

/// Flat list of visible items for the current level (after applying scroll).
#[derive(Debug, Default, Resource)]
pub struct MenuItems {
    pub items: Vec<FlatItem>,
    /// Whether a back button is available (not at root level).
    pub has_back: bool,
    /// Breadcrumb path: "Production > Tris"
    pub breadcrumb: String,
    /// Whether scroll-left is available
    pub can_scroll_left: bool,
    /// Whether scroll-right is available
    pub can_scroll_right: bool,
    /// Total number of items in this level (including scrolled-off)
    pub total_items: usize,
}

/// Number of visible slots (keys 2-0 = 9 slots).
pub const PAGE_SIZE: usize = 9;

// ── TOML types ──

#[derive(Deserialize)]
struct MenuToml {
    menu: Vec<TomlMenuCategory>,
}

#[derive(Deserialize)]
struct TomlMenuCategory {
    label: String,
    items: Vec<TomlMenuItem>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TomlMenuItem {
    String(String),
    SubMenu {
        label: String,
        items: Vec<TomlMenuItem>,
    },
    SpecialAction {
        action: String,
        label: String,
    },
}

// ── Resolve helpers ──

fn resolve_item_id(
    id: &str,
    registry: &BuildingRegistry,
    unit_cfg: &UnitConfig,
) -> Option<MenuEntry> {
    if let Some(def) = registry.buildings.iter().find(|b| b.id == id && !b.hidden) {
        return Some(MenuEntry::Action {
            label: def.name.clone(),
            action: MenuAction::Build(id.to_string()),
        });
    }
    if let Some(def) = unit_cfg.get(id) {
        return Some(MenuEntry::Action {
            label: def.name.clone(),
            action: MenuAction::Spawn(id.to_string()),
        });
    }
    None
}

// ── Loading ──

impl MenuDef {
    pub fn load(registry: &BuildingRegistry, unit_cfg: &UnitConfig) -> Self {
        let toml_str = include_str!("../../data/menu.toml");
        let parsed: MenuToml = toml::from_str(toml_str).expect("failed to parse menu.toml");

        let root = parsed
            .menu
            .into_iter()
            .filter_map(|entry| resolve_root_entry(entry, registry, unit_cfg))
            .collect();

        Self { root }
    }
}

fn resolve_root_entry(
    entry: TomlMenuCategory,
    registry: &BuildingRegistry,
    unit_cfg: &UnitConfig,
) -> Option<MenuEntry> {
    let resolved: Vec<MenuEntry> = entry
        .items
        .into_iter()
        .filter_map(|item| resolve_menu_item(item, registry, unit_cfg))
        .collect();
    if resolved.is_empty() {
        return None;
    }
    Some(MenuEntry::SubMenu {
        label: entry.label,
        items: resolved,
    })
}

fn resolve_menu_item(
    item: TomlMenuItem,
    registry: &BuildingRegistry,
    unit_cfg: &UnitConfig,
) -> Option<MenuEntry> {
    match item {
        TomlMenuItem::String(id) => resolve_item_id(&id, registry, unit_cfg),
        TomlMenuItem::SubMenu { label, items } => {
            let resolved: Vec<MenuEntry> = items
                .into_iter()
                .filter_map(|item| resolve_menu_item(item, registry, unit_cfg))
                .collect();
            if resolved.is_empty() {
                return None;
            }
            Some(MenuEntry::SubMenu {
                label,
                items: resolved,
            })
        }
        TomlMenuItem::SpecialAction { action, label } => {
            let menu_action = match action.as_str() {
                "delete" => MenuAction::Delete,
                _ => return None,
            };
            Some(MenuEntry::Action {
                label,
                action: menu_action,
            })
        }
    }
}

// ── Navigation ──

/// Get the items list at the current path
pub fn items_at<'a>(entries: &'a [MenuEntry], stack: &[usize]) -> &'a [MenuEntry] {
    let mut current = entries;
    for &idx in stack {
        match &current[idx] {
            MenuEntry::SubMenu { items, .. } => current = items,
            _ => return &[],
        }
    }
    current
}

/// Build the breadcrumb string
pub fn breadcrumb_at(entries: &[MenuEntry], stack: &[usize]) -> String {
    let mut current = entries;
    let mut parts = Vec::new();
    for &idx in stack {
        match &current[idx] {
            MenuEntry::SubMenu { label, items } => {
                parts.push(label.clone());
                current = items;
            }
            _ => break,
        }
    }
    parts.join(" > ")
}

/// Build the flat item list for UI, applying scroll.
/// Needs registries to lookup building/unit colors and costs.
pub fn flat_items_at(
    entries: &[MenuEntry],
    stack: &[usize],
    scroll: usize,
    registry: &BuildingRegistry,
    unit_cfg: &UnitConfig,
) -> MenuItems {
    let level = items_at(entries, stack);

    let mut items = Vec::new();
    for entry in level.iter().skip(scroll).take(PAGE_SIZE) {
        match entry {
            MenuEntry::Action { label, action } => {
                let (cost_str, color, texture_stem) = match action {
                    MenuAction::Build(id) => {
                        if let Some(def) = registry.get(id) {
                            (
                                format_cost(&def.cost),
                                def.color,
                                Some(def.texture_stem.clone()),
                            )
                        } else {
                            (String::new(), Color::srgb(0.4, 0.4, 0.5), None)
                        }
                    }
                    MenuAction::Spawn(id) => {
                        if let Some(def) = unit_cfg.get(id) {
                            (
                                format_unit_cost(&def.cost),
                                def.color,
                                Some(def.texture_stem.clone()),
                            )
                        } else {
                            (String::new(), Color::srgb(0.3, 0.35, 0.4), None)
                        }
                    }
                    MenuAction::Delete => (String::new(), Color::srgb(0.8, 0.2, 0.2), None),
                };
                items.push(FlatItem {
                    label: label.clone(),
                    kind: FlatItemKind::Action(action.clone()),
                    cost_str,
                    color,
                    texture_stem,
                });
            }
            MenuEntry::SubMenu { label, .. } => {
                items.push(FlatItem {
                    label: label.clone(),
                    kind: FlatItemKind::SubMenu,
                    cost_str: String::new(),
                    color: Color::srgb(0.4, 0.4, 0.5),
                    texture_stem: None,
                });
            }
        }
    }

    let total_items = level.len();
    let has_back = !stack.is_empty();
    let can_scroll_left = scroll > 0;
    let can_scroll_right = (scroll + PAGE_SIZE) < total_items;
    let breadcrumb = breadcrumb_at(entries, stack);

    MenuItems {
        items,
        has_back,
        breadcrumb,
        can_scroll_left,
        can_scroll_right,
        total_items,
    }
}

fn format_cost(cost: &[crate::economy::building::BuildingCost]) -> String {
    cost.iter()
        .map(|c| format!("{} {:?}", c.amount, c.resource))
        .collect::<Vec<_>>()
        .join(" + ")
}

fn format_unit_cost(cost: &[crate::economy::unit_config::UnitCost]) -> String {
    cost.iter()
        .map(|c| format!("{} {:?}", c.amount, c.resource))
        .collect::<Vec<_>>()
        .join(" + ")
}
