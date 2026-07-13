#![allow(clippy::unnecessary_sort_by)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::type_complexity)]
#![allow(clippy::drop_non_drop)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::useless_format)]
#![allow(clippy::single_match)]
use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::core::modding::ModRegistry;
use crate::ui::components::tooltip::TooltipText;
use crate::core::utils::silent_despawn;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{
    BackButton, BreadcrumbText, BuildMode, DeconstructMode, MenuBarPanel, MenuItemButton,
    Player, ScrollButton,
};
use crate::economy::discovery::GlobalArchive;
use crate::economy::menu::{
    FlatItemKind, MenuAction, MenuDef, MenuEntry, MenuItems, MenuState,
};
use crate::economy::resource::Inventory;
use crate::economy::unit_config::UnitConfig;
use crate::rendering::TextureCache;
use crate::unit::SpawnUnitEvent;

// ── Root marker ──

#[derive(Component)]
pub struct BuildBarRoot;

// ── Config resource (loaded once from panel_build_bar.toml) ──

#[derive(Resource)]
pub struct BuildBarConfig {
    pub item_width: f32,
    pub item_height: f32,
    pub scroll_button_width: f32,
    pub back_button_width: f32,
    pub border_width: f32,
    pub breadcrumb_color: Color,
    pub breadcrumb_font_size: f32,
    pub item_text_color: Color,
    pub key_text_color: Color,
    pub cost_color: Color,
    pub button_bg: Color,
    pub button_border: Color,
    pub scroll_color: Color,
    pub scroll_border: Color,
    pub back_disabled_bg: Color,
    pub back_disabled_border: Color,
    pub back_disabled_text: Color,
}

impl Default for BuildBarConfig {
    fn default() -> Self {
        Self {
            item_width: 90.0,
            item_height: 70.0,
            scroll_button_width: 24.0,
            back_button_width: 60.0,
            border_width: 2.0,
            breadcrumb_color: Color::srgba(0.8, 0.8, 0.9, 0.8),
            breadcrumb_font_size: 12.0,
            item_text_color: Color::WHITE,
            key_text_color: Color::srgba(1.0, 1.0, 1.0, 0.5),
            cost_color: Color::srgb(1.0, 0.85, 0.3),
            button_bg: Color::srgb(0.3, 0.3, 0.4),
            button_border: Color::srgba(1.0, 1.0, 1.0, 0.2),
            scroll_color: Color::srgb(0.25, 0.25, 0.3),
            scroll_border: Color::srgba(1.0, 1.0, 1.0, 0.15),
            back_disabled_bg: Color::srgba(0.2, 0.2, 0.25, 0.5),
            back_disabled_border: Color::srgba(0.3, 0.3, 0.3, 0.3),
            back_disabled_text: Color::srgba(0.5, 0.5, 0.5, 0.5),
        }
    }
}

fn load_build_bar_config(section_config: &toml::Value) -> BuildBarConfig {
    let mut cfg = BuildBarConfig::default();
    if let Some(v) = section_config.get("item_width").and_then(|v| v.as_float()) { cfg.item_width = v as f32; }
    if let Some(v) = section_config.get("item_height").and_then(|v| v.as_float()) { cfg.item_height = v as f32; }
    if let Some(v) = section_config.get("scroll_button_width").and_then(|v| v.as_float()) { cfg.scroll_button_width = v as f32; }
    if let Some(v) = section_config.get("back_button_width").and_then(|v| v.as_float()) { cfg.back_button_width = v as f32; }
    if let Some(v) = section_config.get("border_width").and_then(|v| v.as_float()) { cfg.border_width = v as f32; }
    if let Some(s) = section_config.get("breadcrumb_color").and_then(|v| v.as_str()) { cfg.breadcrumb_color = parse_color(s); }
    if let Some(v) = section_config.get("breadcrumb_font_size").and_then(|v| v.as_float()) { cfg.breadcrumb_font_size = v as f32; }
    if let Some(s) = section_config.get("item_text_color").and_then(|v| v.as_str()) { cfg.item_text_color = parse_color(s); }
    if let Some(s) = section_config.get("key_text_color").and_then(|v| v.as_str()) { cfg.key_text_color = parse_color(s); }
    if let Some(s) = section_config.get("cost_color").and_then(|v| v.as_str()) { cfg.cost_color = parse_color(s); }
    if let Some(s) = section_config.get("button_bg").and_then(|v| v.as_str()) { cfg.button_bg = parse_color(s); }
    if let Some(s) = section_config.get("button_border").and_then(|v| v.as_str()) { cfg.button_border = parse_color(s); }
    if let Some(s) = section_config.get("scroll_color").and_then(|v| v.as_str()) { cfg.scroll_color = parse_color(s); }
    if let Some(s) = section_config.get("scroll_border").and_then(|v| v.as_str()) { cfg.scroll_border = parse_color(s); }
    if let Some(s) = section_config.get("back_disabled_bg").and_then(|v| v.as_str()) { cfg.back_disabled_bg = parse_color(s); }
    if let Some(s) = section_config.get("back_disabled_border").and_then(|v| v.as_str()) { cfg.back_disabled_border = parse_color(s); }
    if let Some(s) = section_config.get("back_disabled_text").and_then(|v| v.as_str()) { cfg.back_disabled_text = parse_color(s); }
    cfg
}

// ── TOML component (renders the container) ──

pub struct BuildBarComponent;
impl crate::ui::registry::UiComponent for BuildBarComponent {
    fn id(&self) -> &str { "build_bar" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &crate::ui::context::UiDataContext, _theme: &crate::ui::theme::Theme, _registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let height = config.get("height").and_then(|v| v.as_float()).unwrap_or(90.0) as f32;
        let bg_color = parse_color(config.get("bg_color").and_then(|v| v.as_str()).unwrap_or("0.1, 0.1, 0.15, 0.85"));
        let padding_x = config.get("padding_x").and_then(|v| v.as_float()).unwrap_or(8.0) as f32;
        let padding_y = config.get("padding_y").and_then(|v| v.as_float()).unwrap_or(4.0) as f32;

        crate::ui::registry::spawn_child(commands, parent, (
            MenuBarPanel,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(height),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                padding: UiRect::axes(Val::Px(padding_x), Val::Px(padding_y)),
                ..default()
            },
            BackgroundColor(bg_color),
            Pickable::default(),
        ))
    }
}

// ── Private helpers ──

fn parse_color(s: &str) -> Color {
    let parts: Vec<f32> = s.split(',').filter_map(|p| p.trim().parse::<f32>().ok()).collect();
    if parts.len() >= 4 {
        Color::srgba(parts[0], parts[1], parts[2], parts[3])
    } else if parts.len() >= 3 {
        Color::srgb(parts[0], parts[1], parts[2])
    } else {
        Color::srgba(0.1, 0.1, 0.15, 0.85)
    }
}

fn slot_key(index: usize) -> &'static str {
    match index {
        0 => "2", 1 => "3", 2 => "4", 3 => "5", 4 => "6",
        5 => "7", 6 => "8", 7 => "9", _ => "0",
    }
}

// ── Content spawning (replaces build_menu_bar from menu.rs) ──

fn spawn_content(commands: &mut Commands, parent: Entity, menu_items: &MenuItems, textures: &TextureCache, cfg: &BuildBarConfig) {
    commands.entity(parent).with_children(|parent| {
        parent.spawn((
            BreadcrumbText,
            Text::new(&menu_items.breadcrumb),
            tf(cfg.breadcrumb_font_size),
            TextColor(cfg.breadcrumb_color),
            Node {
                height: Val::Px(16.0),
                margin: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
        ));

        parent.spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            justify_content: JustifyContent::FlexStart,
            width: Val::Percent(100.0),
            ..default()
        }).with_children(|row| {
            // Back button
            if menu_items.has_back {
                row.spawn((
                    BackButton,
                    Button,
                    Node {
                        width: Val::Px(cfg.back_button_width),
                        height: Val::Px(cfg.item_height),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(cfg.border_width)),
                        ..default()
                    },
                    BackgroundColor(cfg.button_bg),
                    BorderColor::all(cfg.button_border),
                )).with_children(|b| {
                    b.spawn((
                        Text::new("<-1 Retour"),
                        tf(11.0),
                        TextColor(Color::WHITE),
                    ));
                });
            } else {
                row.spawn((
                    Node {
                        width: Val::Px(cfg.back_button_width),
                        height: Val::Px(cfg.item_height),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(cfg.border_width)),
                        ..default()
                    },
                    BackgroundColor(cfg.back_disabled_bg),
                    BorderColor::all(cfg.back_disabled_border),
                )).with_children(|b| {
                    b.spawn((
                        Text::new("1"),
                        tf(11.0),
                        TextColor(cfg.back_disabled_text),
                    ));
                });
            }

            // Left scroll
            if menu_items.can_scroll_left {
                row.spawn((
                    ScrollButton(-1),
                    Button,
                    Node {
                        width: Val::Px(cfg.scroll_button_width),
                        height: Val::Px(cfg.item_height),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(cfg.scroll_color),
                    BorderColor::all(cfg.scroll_border),
                )).with_children(|b| {
                    b.spawn((
                        Text::new("<"),
                        tf(14.0),
                        TextColor(Color::WHITE),
                    ));
                });
            } else {
                row.spawn(Node {
                    width: Val::Px(cfg.scroll_button_width),
                    ..default()
                });
            }

            // Item buttons
            for (i, item) in menu_items.items.iter().enumerate() {
                let key = slot_key(i);
                let sub_prefix = match &item.kind {
                    FlatItemKind::SubMenu => "› ",
                    _ => "",
                };

                row.spawn((
                    MenuItemButton { index: i },
                    Button,
                    Node {
                        width: Val::Px(cfg.item_width),
                        height: Val::Px(cfg.item_height),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(cfg.border_width)),
                        ..default()
                    },
                    BackgroundColor(item.color),
                    BorderColor::all(cfg.button_border),
                )).with_children(|b| {
                    if let Some(stem) = &item.texture_stem
                        && let Some(handle) = textures.base.get(stem) {
                            b.spawn((
                                ImageNode::new(handle.clone()),
                                Node {
                                    width: Val::Px(32.0),
                                    height: Val::Px(32.0),
                                    ..default()
                                },
                            ));
                        }
                    b.spawn((
                        Text::new(format!("{} {}", key, sub_prefix)),
                        tf(9.0),
                        TextColor(cfg.key_text_color),
                    ));
                    b.spawn((
                        Text::new(&item.label),
                        tf(12.0),
                        TextColor(cfg.item_text_color),
                    ));
                    if !item.cost_str.is_empty() {
                        b.spawn((
                            Text::new(&item.cost_str),
                            tf(9.0),
                            TextColor(cfg.cost_color),
                        ));
                    }
                });
            }

            // Right scroll
            if menu_items.can_scroll_right {
                row.spawn((
                    ScrollButton(1),
                    Button,
                    Node {
                        width: Val::Px(cfg.scroll_button_width),
                        height: Val::Px(cfg.item_height),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(cfg.scroll_color),
                    BorderColor::all(cfg.scroll_border),
                )).with_children(|b| {
                    b.spawn((
                        Text::new(">"),
                        tf(14.0),
                        TextColor(Color::WHITE),
                    ));
                });
            } else {
                row.spawn(Node {
                    width: Val::Px(cfg.scroll_button_width),
                    ..default()
                });
            }
        });
    });
}

// ── Lifecycle systems ──

#[allow(clippy::too_many_arguments)]
pub fn spawn_build_bar(
    mut commands: Commands,
    mods: Res<ModRegistry>,
    menu_def: Res<MenuDef>,
    menu_state: Res<MenuState>,
    mut menu_items: ResMut<MenuItems>,
    registry: Res<BuildingRegistry>,
    unit_cfg: Res<UnitConfig>,
    textures: Res<TextureCache>,
    global_archive: Res<GlobalArchive>,
) {
    *menu_items = crate::economy::menu::flat_items_at(
        &menu_def.root, &menu_state.stack, menu_state.scroll,
        &registry, &unit_cfg, &menu_def, &global_archive,
    );

    // Load TOML config
    let Some(content) = mods.load_data("panel_build_bar.toml") else { return };
    let Ok(config) = toml::from_str::<toml::Value>(&content) else { return };

    // Parse and store config resource
    let mut build_bar_cfg = BuildBarConfig::default();
    if let Some(sections) = config.get("sections").and_then(|v| v.as_array()) {
        if let Some(section) = sections.first() {
            build_bar_cfg = load_build_bar_config(section);
        }
    }
    // Get container layout values from TOML
    let section_config = config.get("sections")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first());
    let height = section_config
        .and_then(|v| v.get("height").and_then(|v| v.as_float()))
        .unwrap_or(90.0) as f32;
    let bg_color = parse_color(
        section_config
            .and_then(|v| v.get("bg_color").and_then(|v| v.as_str()))
            .unwrap_or("0.1, 0.1, 0.15, 0.85")
    );
    let padding_x = section_config
        .and_then(|v| v.get("padding_x").and_then(|v| v.as_float()))
        .unwrap_or(8.0) as f32;
    let padding_y = section_config
        .and_then(|v| v.get("padding_y").and_then(|v| v.as_float()))
        .unwrap_or(4.0) as f32;

    // Spawn root + container directly (avoids TOML engine complexity for this special case)
    let root = commands.spawn((
        BuildBarRoot,
        MenuBarPanel,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(height),
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            padding: UiRect::axes(Val::Px(padding_x), Val::Px(padding_y)),
            ..default()
        },
        BackgroundColor(bg_color),
        Pickable::default(),
        ZIndex(100),
    )).id();

    spawn_content(&mut commands, root, &menu_items, &textures, &build_bar_cfg);
    commands.insert_resource(build_bar_cfg);
}

#[allow(clippy::too_many_arguments)]
pub fn refresh_build_bar(
    mut commands: Commands,
    menu_def: Res<MenuDef>,
    menu_state: Res<MenuState>,
    mut menu_items: ResMut<MenuItems>,
    registry: Res<BuildingRegistry>,
    unit_cfg: Res<UnitConfig>,
    global_archive: Res<GlobalArchive>,
    textures: Res<TextureCache>,
    root_q: Query<Entity, With<BuildBarRoot>>,
    build_bar_cfg: Option<Res<BuildBarConfig>>,
) {
    let new_items = crate::economy::menu::flat_items_at(
        &menu_def.root, &menu_state.stack, menu_state.scroll,
        &registry, &unit_cfg, &menu_def, &global_archive,
    );
    if *menu_items == new_items {
        return;
    }
    *menu_items = new_items;

    // Despawn old root complete with all children
    for entity in &root_q {
        silent_despawn(&mut commands, entity);
    }

    let default_cfg = BuildBarConfig::default();
    let cfg = build_bar_cfg.as_deref().unwrap_or(&default_cfg);
    let root = commands.spawn((
        BuildBarRoot,
        MenuBarPanel,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(cfg.item_height),
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.85)),
        Pickable::default(),
        ZIndex(100),
    )).id();

    spawn_content(&mut commands, root, &menu_items, &textures, cfg);
}

pub fn cleanup_build_bar(
    mut commands: Commands,
    root_q: Query<Entity, With<BuildBarRoot>>,
) {
    for e in &root_q { silent_despawn(&mut commands, e); }
}

// ── Keyboard navigation ──

#[allow(clippy::too_many_arguments)]
pub fn menu_navigation(
    mut menu_state: ResMut<MenuState>,
    menu_def: Res<MenuDef>,
    menu_items: Res<MenuItems>,
    mut build_mode: ResMut<BuildMode>,
    mut deconstruct: ResMut<DeconstructMode>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<crate::core::input::KeyBindings>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::Backspace) && !menu_state.stack.is_empty() {
        menu_state.stack.pop();
        menu_state.scroll = 0;
    }

    if keys.just_pressed(KeyCode::Escape) {
        if build_mode.0.is_some() || deconstruct.0 {
            build_mode.0 = None;
            deconstruct.0 = false;
        } else if !menu_state.stack.is_empty() {
            menu_state.stack.pop();
            menu_state.scroll = 0;
        }
    }

    if keys.just_pressed(KeyCode::Digit1) && !menu_state.stack.is_empty() {
        menu_state.stack.pop();
        menu_state.scroll = 0;
    }

    let digit_keys = [
        KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5,
        KeyCode::Digit6, KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9, KeyCode::Digit0,
    ];
    for (slot, key) in digit_keys.iter().enumerate() {
        if keys.just_pressed(*key)
            && let Some(item) = menu_items.items.get(slot) {
                activate_item(item, &mut menu_state, &menu_def, &mut build_mode, &mut deconstruct, &mut commands);
            }
    }

    if bindings.just_pressed("build_deconstruct", &keys, &mouse) {
        if build_mode.0.is_some() {
            build_mode.0 = None;
        }
        deconstruct.0 = !deconstruct.0;
    }
}

// ── Mouse interaction ──

#[allow(clippy::too_many_arguments)]
pub fn menu_bar_interaction(
    query: Query<(&Interaction, &MenuItemButton), Changed<Interaction>>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>,
    scroll_query: Query<(&Interaction, &ScrollButton), Changed<Interaction>>,
    menu_items: Res<MenuItems>,
    mut menu_state: ResMut<MenuState>,
    menu_def: Res<MenuDef>,
    mut build_mode: ResMut<BuildMode>,
    mut deconstruct: ResMut<DeconstructMode>,
    registry: Res<BuildingRegistry>,
    unit_cfg: Res<UnitConfig>,
    mut commands: Commands,
    mut tooltip: ResMut<TooltipText>,
) {
    for (interaction, button) in &query {
        if *interaction == Interaction::Pressed
            && let Some(item) = menu_items.items.get(button.index) {
                activate_item(item, &mut menu_state, &menu_def, &mut build_mode, &mut deconstruct, &mut commands);
            }
        if *interaction == Interaction::Hovered
            && let Some(item) = menu_items.items.get(button.index) {
                tooltip.0 = Some(match &item.kind {
                    FlatItemKind::Action(action) => match action {
                        MenuAction::Build(id) => registry
                            .get(id)
                            .map(|def| {
                                let mut parts = vec![format!(
                                    "{}  HP:{}  Cost:{}",
                                    def.name, def.hp, item.cost_str
                                )];
                                if def.requires_deposit {
                                    parts.push("Requires ore deposit".into());
                                }
                                if let Some(ref p) = def.production {
                                    parts.push(format!(
                                        "Produces {} every {:.1}s",
                                        p.resource.display_name(),
                                        p.interval_sec
                                    ));
                                }
                                if let Some(ref b) = def.belt {
                                    parts.push(format!("{} slots, speed {:.1}", b.slots, b.speed));
                                }
                                if let Some(ref c) = def.combat {
                                    parts.push(format!(
                                        "Dmg {}  Range {:.0}  Rate {:.1}s",
                                        c.damage, c.range.sqrt(), c.fire_rate_sec
                                    ));
                                }
                                parts.join("  |  ")
                            })
                            .unwrap_or_default(),
                        MenuAction::Spawn(id) => unit_cfg
                            .get(id)
                            .map(|def| {
                                let mut parts = vec![format!(
                                    "{}  HP:{}  Cost:{}",
                                    def.name, def.hp, item.cost_str
                                )];
                                if def.kind == "combat" {
                                    parts.push(format!(
                                        "Dmg {}  Range {:.0}  Rate {:.1}s",
                                        def.damage, def.range_tiles, def.fire_rate_sec
                                    ));
                                } else if def.kind == "harvester" {
                                    parts.push(format!(
                                        "Speed {:.0}  Mine interval {:.1}s",
                                        def.speed, def.mine_interval_sec
                                    ));
                                }
                                parts.join("  |  ")
                            })
                            .unwrap_or_default(),
                        MenuAction::Delete => {
                            "[Delete] Deconstruct mode — click a building to dismantle".into()
                        }
                    },
                    FlatItemKind::SubMenu => format!("{} › (click to enter)", item.label),
                });
            }
        if *interaction == Interaction::None {
            tooltip.0 = None;
        }
    }

    if let Ok(interaction) = back_query.single()
        && *interaction == Interaction::Pressed && !menu_state.stack.is_empty() {
            menu_state.stack.pop();
            menu_state.scroll = 0;
        }

    for (interaction, scroll) in &scroll_query {
        if *interaction == Interaction::Pressed {
            let max = menu_items.total_items.saturating_sub(menu_def.page_size);
            if scroll.0 < 0 {
                menu_state.scroll = menu_state.scroll.saturating_sub(1);
            } else if menu_state.scroll < max {
                menu_state.scroll += 1;
            }
        }
    }
}

// ── Visual update (affordability + active state) ──

#[allow(clippy::too_many_arguments)]
pub fn update_menu_bar(
    build_mode: Res<BuildMode>,
    deconstruct: Res<DeconstructMode>,
    player_query: Query<&Inventory, With<Player>>,
    registry: Res<BuildingRegistry>,
    unit_cfg: Res<UnitConfig>,
    menu_items: Res<MenuItems>,
    mut button_query: Query<(&MenuItemButton, &mut BackgroundColor, &mut BorderColor)>,
) {
    let player_inv = player_query.single().ok();
    let has_build_mode = build_mode.0.is_some();
    let has_deconstruct = deconstruct.0;

    for (button, mut bg, mut border) in button_query.iter_mut() {
        let Some(item) = menu_items.items.get(button.index) else {
            continue;
        };
        match &item.kind {
            FlatItemKind::Action(action) => match action {
                MenuAction::Build(id) => {
                    let is_active = build_mode.0.as_ref() == Some(id);
                    let affordable = player_inv
                        .and_then(|inv| {
                            registry.get(id).map(|def| {
                                def.cost.iter().all(|c| inv.get(&c.resource) >= c.amount)
                            })
                        })
                        .unwrap_or(false);

                    *border = BorderColor::all(if is_active {
                        Color::srgb(0.3, 1.0, 0.3)
                    } else if has_deconstruct || (has_build_mode && !is_active) {
                        Color::srgba(0.3, 0.3, 0.3, 0.3)
                    } else {
                        Color::srgba(1.0, 1.0, 1.0, 0.2)
                    });
                    bg.0 = if affordable {
                        item.color
                    } else {
                        Color::srgb(0.3, 0.3, 0.3)
                    };
                }
                MenuAction::Spawn(id) => {
                    let affordable = player_inv
                        .and_then(|inv| {
                            unit_cfg.get(id).map(|def| {
                                def.cost.iter().all(|c| inv.get(&c.resource) >= c.amount)
                            })
                        })
                        .unwrap_or(false);
                    bg.0 = if affordable {
                        item.color
                    } else {
                        Color::srgb(0.3, 0.3, 0.3)
                    };
                }
                MenuAction::Delete => {
                    *border = BorderColor::all(if deconstruct.0 {
                        Color::srgb(0.3, 1.0, 0.3)
                    } else {
                        Color::srgba(1.0, 1.0, 1.0, 0.2)
                    });
                    bg.0 = item.color;
                }
            },
            FlatItemKind::SubMenu => {
                bg.0 = item.color;
            }
        }
    }
}

// ── Activation logic (shared between keyboard + mouse) ──

fn activate_item(
    item: &crate::economy::menu::FlatItem,
    menu_state: &mut MenuState,
    menu_def: &MenuDef,
    build_mode: &mut BuildMode,
    deconstruct: &mut DeconstructMode,
    commands: &mut Commands,
) {
    match &item.kind {
        FlatItemKind::SubMenu => {
            let level = crate::economy::menu::items_at(&menu_def.root, &menu_state.stack);
            let idx = level.iter().position(|e| match e {
                MenuEntry::SubMenu { label, .. } => label == &item.label,
                _ => false,
            });
            if let Some(idx) = idx {
                menu_state.stack.push(idx);
                menu_state.scroll = 0;
                let new_level = crate::economy::menu::items_at(&menu_def.root, &menu_state.stack);
                if let Some(MenuEntry::Action {
                    action: first_action, ..
                }) = new_level.first()
                {
                    match first_action {
                        MenuAction::Build(id) => {
                            build_mode.0 = Some(id.clone());
                        }
                        MenuAction::Spawn(id) => {
                            commands.trigger(SpawnUnitEvent(id.clone()));
                        }
                        _ => {}
                    }
                }
            }
        }
        FlatItemKind::Action(action) => match action {
            MenuAction::Build(id) => {
                deconstruct.0 = false;
                build_mode.0 = match &build_mode.0 {
                    Some(current) if current == id => None,
                    _ => Some(id.clone()),
                };
            }
            MenuAction::Spawn(id) => {
                commands.trigger(SpawnUnitEvent(id.clone()));
            }
            MenuAction::Delete => {
                deconstruct.0 = !deconstruct.0;
                if deconstruct.0 {
                    build_mode.0 = None;
                }
            }
        },
    }
}
