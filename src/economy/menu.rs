// Menu data types (items, categories, breadcrumb) loaded from TOML menus.

use crate::economy::building::BuildingRegistry;
use crate::economy::discovery::GlobalArchive;
use crate::economy::resource::Cost;
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
    pub page_size: usize,
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

impl PartialEq for MenuItems {
    fn eq(&self, other: &Self) -> bool {
        self.items.len() == other.items.len()
            && self.items.iter().zip(other.items.iter()).all(|(a, b)| {
                a.label == b.label
                    && std::mem::discriminant(&a.kind) == std::mem::discriminant(&b.kind)
            })
            && self.has_back == other.has_back
            && self.breadcrumb == other.breadcrumb
            && self.can_scroll_left == other.can_scroll_left
            && self.can_scroll_right == other.can_scroll_right
            && self.total_items == other.total_items
    }
}

/// Number of visible slots (keys 2-0 = 9 slots).

// ── TOML types ──

#[derive(Deserialize)]
struct MenuToml {
    #[serde(default)]
    settings: MenuSettings,
    menu: Vec<TomlMenuCategory>,
}

#[derive(Default, Deserialize)]
struct MenuSettings {
    #[serde(default = "default_page_size")]
    page_size: usize,
}

fn default_page_size() -> usize {
    9
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
    pub fn load(
        mods: &crate::core::modding::ModRegistry,
        registry: &BuildingRegistry,
        unit_cfg: &UnitConfig,
    ) -> Self {
        let parsed: MenuToml = mods.load_toml("menu.toml");
        let page_size = parsed.settings.page_size;

        let root = parsed
            .menu
            .into_iter()
            .filter_map(|entry| resolve_root_entry(entry, registry, unit_cfg))
            .collect();

        Self { root, page_size }
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

fn has_visible_children(
    entry: &MenuEntry,
    registry: &BuildingRegistry,
    unit_cfg: &UnitConfig,
    global_archive: &GlobalArchive,
) -> bool {
    match entry {
        MenuEntry::Action { action, .. } => match action {
            MenuAction::Build(id) => registry.get(id).is_some_and(|def| {
                if def.hidden {
                    return false;
                }
                if let Some(ref req) = def.requires_discovery
                    && !global_archive.is_unlocked(req) {
                        return false;
                    }
                true
            }),
            MenuAction::Spawn(id) => unit_cfg.get(id).is_some(),
            MenuAction::Delete => true,
        },
        MenuEntry::SubMenu { items, .. } => items
            .iter()
            .any(|child| has_visible_children(child, registry, unit_cfg, global_archive)),
    }
}

/// Build the flat item list for UI, applying scroll.
/// Needs registries to lookup building/unit colors and costs.
pub fn flat_items_at(
    entries: &[MenuEntry],
    stack: &[usize],
    scroll: usize,
    registry: &BuildingRegistry,
    unit_cfg: &UnitConfig,
    menu_def: &MenuDef,
    global_archive: &GlobalArchive,
) -> MenuItems {
    let level = items_at(entries, stack);

    let mut items = Vec::new();
    for entry in level.iter() {
        match entry {
            MenuEntry::Action { label, action } => match action {
                MenuAction::Build(id) => {
                    if let Some(def) = registry.get(id) {
                        if def.hidden {
                            continue;
                        }
                        if let Some(ref req) = def.requires_discovery
                            && !global_archive.is_unlocked(req) {
                                continue;
                            }
                        items.push(FlatItem {
                            label: label.clone(),
                            kind: FlatItemKind::Action(action.clone()),
                            cost_str: format_cost(&def.cost),
                            color: def.color,
                            texture_stem: Some(def.texture_stem.clone()),
                        });
                    }
                }
                MenuAction::Spawn(id) => {
                    if let Some(def) = unit_cfg.get(id) {
                        items.push(FlatItem {
                            label: label.clone(),
                            kind: FlatItemKind::Action(action.clone()),
                            cost_str: format_cost(&def.cost),
                            color: def.color,
                            texture_stem: Some(def.texture_stem.clone()),
                        });
                    }
                }
                MenuAction::Delete => {
                    items.push(FlatItem {
                        label: label.clone(),
                        kind: FlatItemKind::Action(action.clone()),
                        cost_str: String::new(),
                        // ⚠️ IA ATTENTION: couleur de l'action en dur (rouge).
                        color: Color::srgb(0.8, 0.2, 0.2),
                        texture_stem: None,
                    });
                }
            },
            MenuEntry::SubMenu { label, items: children } => {
                if children
                    .iter()
                    .any(|child| has_visible_children(child, registry, unit_cfg, global_archive))
                {
                    items.push(FlatItem {
                        label: label.clone(),
                        kind: FlatItemKind::SubMenu,
                        cost_str: String::new(),
                        // ⚠️ IA ATTENTION: couleur de sous-menu en dur (gris).
                        color: Color::srgb(0.4, 0.4, 0.5),
                        texture_stem: None,
                    });
                }
            }
        }
    }

    let total_before_scroll = items.len();
    let page_items: Vec<FlatItem> = items
        .into_iter()
        .skip(scroll)
        .take(menu_def.page_size)
        .collect();
    let has_back = !stack.is_empty();
    let can_scroll_left = scroll > 0;
    let can_scroll_right = (scroll + menu_def.page_size) < total_before_scroll;
    let breadcrumb = breadcrumb_at(entries, stack);

    MenuItems {
        items: page_items,
        has_back,
        breadcrumb,
        can_scroll_left,
        can_scroll_right,
        total_items: total_before_scroll,
    }
}

pub fn format_cost(cost: &[Cost]) -> String {
    cost.iter()
        .map(|c| format!("{} {:?}", c.amount, c.resource))
        .collect::<Vec<_>>()
        .join(" + ")
}



