use crate::core::tooltip::TooltipText;
use crate::core::utils::silent_despawn;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{
    BackButton, BreadcrumbText, BuildMode, DeconstructMode, MenuBarPanel, MenuItemButton, Player,
    ScrollButton,
};
use crate::economy::menu::{
    FlatItemKind, MenuAction, MenuDef, MenuEntry, MenuItems, MenuState, PAGE_SIZE,
};
use crate::economy::resource::Inventory;
use crate::economy::unit_config::UnitConfig;
use crate::rendering::TextureCache;
use crate::unit::SpawnUnitEvent;
use bevy::prelude::*;

const PANEL_HEIGHT: f32 = 90.0;
const ITEM_WIDTH: f32 = 90.0;
const ITEM_HEIGHT: f32 = 70.0;

fn slot_key(index: usize) -> &'static str {
    match index {
        0 => "2",
        1 => "3",
        2 => "4",
        3 => "5",
        4 => "6",
        5 => "7",
        6 => "8",
        7 => "9",
        _ => "0",
    }
}

// ── Spawn ──

pub fn spawn_menu_bar(
    mut commands: Commands,
    menu_def: Res<MenuDef>,
    menu_state: Res<MenuState>,
    mut menu_items: ResMut<MenuItems>,
    registry: Res<BuildingRegistry>,
    unit_cfg: Res<UnitConfig>,
    textures: Res<TextureCache>,
) {
    *menu_items = crate::economy::menu::flat_items_at(
        &menu_def.root,
        &menu_state.stack,
        menu_state.scroll,
        &registry,
        &unit_cfg,
    );

    build_menu_bar(&mut commands, &menu_items, &textures);
}

// ── Rebuild MenuItems + refresh UI when state changes ──

pub fn refresh_menu_bar(
    mut commands: Commands,
    menu_def: Res<MenuDef>,
    menu_state: Res<MenuState>,
    mut menu_items: ResMut<MenuItems>,
    registry: Res<BuildingRegistry>,
    unit_cfg: Res<UnitConfig>,
    textures: Res<TextureCache>,
    panel_query: Query<Entity, With<MenuBarPanel>>,
) {
    let new_items = crate::economy::menu::flat_items_at(
        &menu_def.root,
        &menu_state.stack,
        menu_state.scroll,
        &registry,
        &unit_cfg,
    );
    if *menu_items == new_items {
        return;
    }
    *menu_items = new_items;

    for entity in &panel_query {
        silent_despawn(&mut commands, entity);
    }
    build_menu_bar(&mut commands, &menu_items, &textures);
}

// ── Shared panel builder (called on create AND refresh) ──

fn build_menu_bar(commands: &mut Commands, menu_items: &MenuItems, textures: &TextureCache) {
    commands
        .spawn((
            MenuBarPanel,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(PANEL_HEIGHT),
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
        ))
        .with_children(|parent| {
            // Breadcrumb
            parent.spawn((
                BreadcrumbText,
                Text::new(&menu_items.breadcrumb),
                TextFont::from_font_size(12.0),
                TextColor(Color::srgba(0.8, 0.8, 0.9, 0.8)),
                Node {
                    height: Val::Px(16.0),
                    margin: UiRect::bottom(Val::Px(2.0)),
                    ..default()
                },
            ));

            // Item row
            parent
                .spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.0),
                    justify_content: JustifyContent::FlexStart,
                    width: Val::Percent(100.0),
                    ..default()
                })
                .with_children(|row| {
                    // Back or spacer
                    if menu_items.has_back {
                        row.spawn((
                            BackButton,
                            Button,
                            Node {
                                width: Val::Px(60.0),
                                height: Val::Px(ITEM_HEIGHT),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("<-1 Retour"),
                                TextFont::from_font_size(11.0),
                                TextColor(Color::WHITE),
                            ));
                        });
                    } else {
                        row.spawn((
                            Node {
                                width: Val::Px(60.0),
                                height: Val::Px(ITEM_HEIGHT),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.5)),
                            BorderColor::all(Color::srgba(0.3, 0.3, 0.3, 0.3)),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("1"),
                                TextFont::from_font_size(11.0),
                                TextColor(Color::srgba(0.5, 0.5, 0.5, 0.5)),
                            ));
                        });
                    }

                    // Scroll left
                    if menu_items.can_scroll_left {
                        row.spawn((
                            ScrollButton(-1),
                            Button,
                            Node {
                                width: Val::Px(24.0),
                                height: Val::Px(ITEM_HEIGHT),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.15)),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("<"),
                                TextFont::from_font_size(14.0),
                                TextColor(Color::WHITE),
                            ));
                        });
                    } else {
                        row.spawn(Node {
                            width: Val::Px(24.0),
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
                        let bg_color = item.color;

                        row.spawn((
                            MenuItemButton { index: i },
                            Button,
                            Node {
                                width: Val::Px(ITEM_WIDTH),
                                height: Val::Px(ITEM_HEIGHT),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(bg_color),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                        ))
                        .with_children(|b| {
                            // Sprite preview
                            if let Some(stem) = &item.texture_stem {
                                if let Some(handle) = textures.base.get(stem) {
                                    b.spawn((
                                        ImageNode::new(handle.clone()),
                                        Node {
                                            width: Val::Px(32.0),
                                            height: Val::Px(32.0),
                                            ..default()
                                        },
                                    ));
                                }
                            }
                            b.spawn((
                                Text::new(format!("{} {}", key, sub_prefix)),
                                TextFont::from_font_size(9.0),
                                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                            ));
                            b.spawn((
                                Text::new(&item.label),
                                TextFont::from_font_size(12.0),
                                TextColor(Color::WHITE),
                            ));
                            if !item.cost_str.is_empty() {
                                b.spawn((
                                    Text::new(&item.cost_str),
                                    TextFont::from_font_size(9.0),
                                    TextColor(Color::srgb(1.0, 0.85, 0.3)),
                                ));
                            }
                        });
                    }

                    // Scroll right
                    if menu_items.can_scroll_right {
                        row.spawn((
                            ScrollButton(1),
                            Button,
                            Node {
                                width: Val::Px(24.0),
                                height: Val::Px(ITEM_HEIGHT),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.15)),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new(">"),
                                TextFont::from_font_size(14.0),
                                TextColor(Color::WHITE),
                            ));
                        });
                    } else {
                        row.spawn(Node {
                            width: Val::Px(24.0),
                            ..default()
                        });
                    }
                });
        });
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

// ── Navigation (keyboard) ──

pub fn menu_navigation(
    mut menu_state: ResMut<MenuState>,
    menu_def: Res<MenuDef>,
    menu_items: Res<MenuItems>,
    mut build_mode: ResMut<BuildMode>,
    mut deconstruct: ResMut<DeconstructMode>,
    keys: Res<ButtonInput<KeyCode>>,
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
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
        KeyCode::Digit0,
    ];
    for (slot, key) in digit_keys.iter().enumerate() {
        if keys.just_pressed(*key) {
            if let Some(item) = menu_items.items.get(slot) {
                activate_item(
                    item,
                    &mut menu_state,
                    &menu_def,
                    &mut build_mode,
                    &mut deconstruct,
                    &mut commands,
                );
            }
        }
    }

    if keys.just_pressed(bindings.key("build_deconstruct")) {
        if build_mode.0.is_some() {
            build_mode.0 = None;
        }
        deconstruct.0 = !deconstruct.0;
    }
}

// ── Interactions (mouse clicks) ──

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
        if *interaction == Interaction::Pressed {
            if let Some(item) = menu_items.items.get(button.index) {
                activate_item(
                    item,
                    &mut menu_state,
                    &menu_def,
                    &mut build_mode,
                    &mut deconstruct,
                    &mut commands,
                );
            }
        }
        if *interaction == Interaction::Hovered {
            if let Some(item) = menu_items.items.get(button.index) {
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
                                        c.damage,
                                        c.range.sqrt(),
                                        c.fire_rate_sec
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
        }
        if *interaction == Interaction::None {
            tooltip.0 = None;
        }
    }

    if let Ok(interaction) = back_query.single() {
        if *interaction == Interaction::Pressed && !menu_state.stack.is_empty() {
            menu_state.stack.pop();
            menu_state.scroll = 0;
        }
    }

    for (interaction, scroll) in &scroll_query {
        if *interaction == Interaction::Pressed {
            let max = if menu_items.total_items > PAGE_SIZE {
                menu_items.total_items - PAGE_SIZE
            } else {
                0
            };
            if scroll.0 < 0 {
                menu_state.scroll = menu_state.scroll.saturating_sub(1);
            } else if menu_state.scroll < max {
                menu_state.scroll += 1;
            }
        }
    }
}

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
                // Auto-select first item in the submenu
                let new_level = crate::economy::menu::items_at(&menu_def.root, &menu_state.stack);
                if let Some(MenuEntry::Action {
                    action: first_action,
                    ..
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

// ── Update (colors, affordability) ──

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

// ── Cleanup ──

pub fn cleanup_menu_bar(mut commands: Commands, query: Query<Entity, With<MenuBarPanel>>) {
    for entity in &query {
        silent_despawn(&mut commands, entity);
    }
}
