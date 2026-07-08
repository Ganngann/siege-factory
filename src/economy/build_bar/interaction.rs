use crate::core::tooltip::TooltipText;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{
    BackButton, BuildMode, DeconstructMode, MenuItemButton, Player, ScrollButton,
};
use crate::economy::menu::{FlatItemKind, MenuAction, MenuDef, MenuEntry, MenuItems, MenuState};
use crate::economy::resource::Inventory;
use crate::economy::unit_config::UnitConfig;
use crate::unit::SpawnUnitEvent;
use bevy::prelude::*;

const UNIT_KIND_COMBAT: &str = "combat";
const UNIT_KIND_HARVESTER: &str = "harvester";

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
        if keys.just_pressed(*key)
            && let Some(item) = menu_items.items.get(slot) {
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

    if bindings.just_pressed("build_deconstruct", &keys, &mouse) {
        if build_mode.0.is_some() {
            build_mode.0 = None;
        }
        deconstruct.0 = !deconstruct.0;
    }
}

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
                activate_item(
                    item,
                    &mut menu_state,
                    &menu_def,
                    &mut build_mode,
                    &mut deconstruct,
                    &mut commands,
                );
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
                                if def.kind == UNIT_KIND_COMBAT {
                                    parts.push(format!(
                                        "Dmg {}  Range {:.0}  Rate {:.1}s",
                                        def.damage, def.range_tiles, def.fire_rate_sec
                                    ));
                                } else if def.kind == UNIT_KIND_HARVESTER {
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
