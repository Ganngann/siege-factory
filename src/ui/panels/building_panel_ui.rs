use bevy::prelude::*;

use crate::core::game_font::tf;
use crate::economy::building::{BuildingDef, BuildingRegistry};
use crate::economy::components::{InventoryGrid, InventorySlot};
use crate::economy::resource::ResourceRegistry;
use crate::economy::ui_components::*;
use crate::economy::window::*;

const BUILDING_KIND_SORTER: &str = "sorter";

/// Construit l'UI complète du panneau d'inspection d'un bâtiment.
/// Retourne l'entité racine de la fenêtre.
pub fn build_building_panel_ui(
    commands: &mut Commands,
    modal_size: Vec2,
    entity: Entity,
    kind: &str,
    show_recipes: bool,
    is_farm: bool,
    resource_registry: &ResourceRegistry,
    reg: &BuildingRegistry,
    farm_crop_types: Vec<String>,
) -> Entity {
    let panel_w = modal_size.x;
    let col_w = panel_w * 0.58;
    let right_w = panel_w * 0.38;
    let show_sorter = kind == BUILDING_KIND_SORTER;

    let x = (1280.0 - modal_size.x) / 2.0;
    let y = (720.0 - modal_size.y) / 2.0;

    spawn_window(
        commands,
        &format!("{}  #{}", kind, entity.to_bits() % 1000),
        modal_size.x,
        modal_size.y,
        x,
        y,
        None,
        |parent| {
            spawn_status_bar(parent);

            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },))
                .with_children(|row| {
                    row.spawn((Node {
                        width: Val::Px(col_w),
                        flex_direction: FlexDirection::Column,
                        margin: UiRect::right(Val::Px(10.0)),
                        ..default()
                    },))
                        .with_children(|left| {
                            spawn_section(left, "FLOW", spawn_flow_content);
                            spawn_section(left, "INVENTORY", |sec| {
                                spawn_inventory_content(sec, entity);
                            });
                            spawn_section(left, "CONNECTIONS", spawn_connections_content);
                        });

                    row.spawn((Node {
                        width: Val::Px(right_w),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },))
                        .with_children(|right| {
                            spawn_section(right, "STATS", spawn_stats_content);
                            spawn_section(right, "POWER", spawn_power_content);

                            let is_burner = reg
                                .get(kind)
                                .map(|d| d.fuel_burn_interval > 0.0)
                                .unwrap_or(false);
                            if is_burner {
                                spawn_section(right, "FUEL", spawn_burner_content);
                            }

                            if show_recipes {
                                spawn_section(right, "SETTINGS", spawn_recipe_content);
                            }

                            if is_farm {
                                spawn_section(right, "FARM", |sec| {
                                    spawn_farm_content(sec, farm_crop_types);
                                });
                            }

                            if show_sorter {
                                spawn_section(right, "FILTER", |sec| {
                                    spawn_sorter_content(sec, resource_registry);
                                });
                            }

                            spawn_section(right, "HP", spawn_hp_content);
                            spawn_section(right, "ALERTS", spawn_alerts_content);

                            if let Some(upgrade_kind) =
                                reg.get(kind).and_then(|d| d.upgrades_to.as_ref())
                                && let Some(upgrade_def) = reg.get(upgrade_kind) {
                                    spawn_upgrade_section(right, upgrade_kind, upgrade_def);
                                }
                        });
                });
        },
    )
}

// ── Status bar ──

fn spawn_status_bar(parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(BG_SECTION),
        ))
        .with_children(|status| {
            status
                .spawn((
                    ProgressBarBg,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(14.0),
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    BackgroundColor(BAR_BG),
                ))
                .with_children(|bg| {
                    bg.spawn((
                        ProgressBarFill,
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(ACCENT),
                    ));
                });
            status.spawn((
                StatusText,
                Text::new("Idle"),
                tf(12.0),
                TextColor(TEXT_SECONDARY),
                Node {
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
            ));
        });
}

// ── Section helpers ──

fn spawn_flow_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
    sec.spawn((
        FlowInputText,
        Text::new("Inputs: --"),
        tf(12.0),
        TextColor(TEXT_SECONDARY),
        Node { margin: UiRect::bottom(Val::Px(2.0)), ..default() },
    ));
    sec.spawn((
        FlowOutputText,
        Text::new("Outputs: --"),
        tf(12.0),
        TextColor(TEXT_SECONDARY),
    ));
}

fn spawn_inventory_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands, entity: Entity) {
    sec.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(12.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        BackgroundColor(BAR_BG),
    ))
    .with_children(|bar| {
        bar.spawn((
            CapacityBarFill,
            Node {
                width: Val::Percent(0.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.30, 0.55, 0.30)),
        ));
    });
    sec.spawn((
        CapacityBarText,
        Text::new("Capacity: 0/0"),
        tf(11.0),
        TextColor(TEXT_SECONDARY),
        Node { margin: UiRect::vertical(Val::Px(4.0)), ..default() },
    ));
    const S2: f32 = 40.0;
    const G2: f32 = 3.0;
    sec.spawn((
        InventoryGrid { cols: 3, rows: 2, owner: entity },
        Node {
            width: Val::Px(3.0 * (S2 + G2) + G2 * 2.0),
            padding: bevy::ui::UiRect::all(Val::Px(G2)),
            display: Display::Flex,
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::FlexStart,
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9)),
    ))
    .with_children(|g| {
        for i in 0..6 {
            g.spawn((
                InventorySlot { index: i },
                Button,
                Node {
                    width: Val::Px(S2), height: Val::Px(S2),
                    flex_shrink: 0.0,
                    margin: bevy::ui::UiRect::axes(Val::Px(G2 / 2.0), Val::Px(G2 / 2.0)),
                    border: bevy::ui::UiRect::all(Val::Px(1.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BorderColor::all(Color::srgba(0.3, 0.3, 0.4, 1.0)),
                BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 1.0)),
                Text::new(String::new()),
                tf(9.0),
                TextColor(Color::WHITE),
            ));
        }
    });
}

fn spawn_connections_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
    sec.spawn((
        ConnectionRowText,
        Text::new("No connections"),
        tf(11.0),
        TextColor(TEXT_SECONDARY),
    ));
}

fn spawn_stats_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
    for line in [
        "Produced/min:  --",
        "Consumed/min:  --",
        "Uptime:        --",
        "Efficiency:    --",
        "Total output:  0",
    ] {
        sec.spawn((
            StatRowText,
            Text::new(line),
            tf(11.0),
            TextColor(TEXT_SECONDARY),
            Node { margin: UiRect::bottom(Val::Px(1.0)), ..default() },
        ));
    }
}

fn spawn_power_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
    sec.spawn((
        PowerStatusText,
        Text::new("Power: --"),
        tf(11.0),
        TextColor(TEXT_SECONDARY),
    ));
}

fn spawn_recipe_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
    sec.spawn((
        RecipeNameText,
        Text::new("Recipe: --"),
        tf(12.0),
        TextColor(TEXT_PRIMARY),
        Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
    ));
    sec.spawn((
        RecipeChangeButton,
        Button,
        Node {
            width: Val::Px(120.0), height: Val::Px(24.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.18, 0.25, 0.40)),
    ))
    .with_children(|btn| {
        btn.spawn((
            Text::new("[Change Recipe]"),
            tf(11.0),
            TextColor(ACCENT),
        ));
    });
}

fn spawn_farm_content(
    sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    farm_crop_types: Vec<String>,
) {
    sec.spawn((
        FarmCropText,
        Text::new("Crops:  --"),
        tf(12.0),
        TextColor(TEXT_PRIMARY),
        Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
    ));
    for crop_type in &farm_crop_types {
        sec.spawn((
            FarmCropSelectButton { crop_type: crop_type.clone() },
            Button,
            Node {
                width: Val::Px(120.0), height: Val::Px(22.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::vertical(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.18, 0.35, 0.18)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(crop_type),
                tf(11.0),
                TextColor(TEXT_SECONDARY),
            ));
        });
    }
    sec.spawn((
        FarmCultivatorCountText,
        Text::new("Cultivators:  --"),
        tf(12.0),
        TextColor(TEXT_SECONDARY),
        Node { margin: UiRect::vertical(Val::Px(4.0)), ..default() },
    ));
    sec.spawn((
        FarmRecruitButton,
        Button,
        Node {
            width: Val::Px(160.0), height: Val::Px(26.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.25, 0.55, 0.25)),
    ))
    .with_children(|btn| {
        btn.spawn((
            Text::new("[Recruit Cultivator]  8 ore"),
            tf(12.0),
            TextColor(TEXT_PRIMARY),
        ));
    });
}

fn spawn_sorter_content(
    sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    resource_registry: &ResourceRegistry,
) {
    sec.spawn((
        SorterInvertButton,
        Button,
        Node {
            width: Val::Percent(100.0), height: Val::Px(26.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.30, 0.30, 0.15)),
    ))
    .with_children(|btn| {
        btn.spawn((
            Text::new("Invert: OFF"),
            tf(12.0),
            TextColor(TEXT_PRIMARY),
        ));
    });

    let mut resources: Vec<String> = resource_registry.resources.keys().cloned().collect();
    resources.sort();
    for res in &resources {
        sec.spawn((
            SorterResourceButton { resource: crate::economy::resource::ResourceId(res.clone()) },
            Button,
            Node {
                width: Val::Percent(100.0), height: Val::Px(22.0),
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(8.0)),
                margin: UiRect::vertical(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.22)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(res),
                tf(11.0),
                TextColor(TEXT_SECONDARY),
            ));
        });
    }
}

fn spawn_hp_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
    sec.spawn((
        Node {
            width: Val::Percent(100.0), height: Val::Px(12.0),
            ..default()
        },
        BackgroundColor(BAR_BG),
    ))
    .with_children(|bar| {
        bar.spawn((
            HpBarFill,
            Node {
                width: Val::Percent(100.0), height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(HP_GREEN),
        ));
    });
    sec.spawn((
        HpText,
        Text::new("HP: --/--"),
        tf(11.0),
        TextColor(TEXT_SECONDARY),
        Node { margin: UiRect::top(Val::Px(4.0)), ..default() },
    ));
}

fn spawn_alerts_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
    sec.spawn((
        AlertText,
        Text::new("No alerts"),
        tf(11.0),
        TextColor(TEXT_SECONDARY),
    ));
}

fn spawn_upgrade_section(
    sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    target_kind: &str,
    target_def: &BuildingDef,
) {
    let cost_str: String = target_def
        .cost.iter()
        .map(|c| format!("{} x{}", c.resource.0, c.amount))
        .collect::<Vec<_>>()
        .join(", ");

    sec.spawn((
        UpgradeInfoText,
        Text::new(format!("Upgrade to: {}", target_def.name)),
        tf(12.0),
        TextColor(TEXT_PRIMARY),
        Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
    ));
    sec.spawn((
        Text::new(format!("Cost: {}", cost_str)),
        tf(11.0),
        TextColor(TEXT_SECONDARY),
        Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
    ));
    sec.spawn((
        UpgradeButton { target_kind: target_kind.to_string() },
        Button,
        Node {
            width: Val::Px(160.0), height: Val::Px(26.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.25, 0.45, 0.65)),
    ))
    .with_children(|btn| {
        btn.spawn((
            Text::new("[Upgrade]"),
            tf(12.0),
            TextColor(TEXT_PRIMARY),
        ));
    });
}

fn spawn_burner_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
    sec.spawn((
        Text::new("Combustion"),
        tf(10.0),
        TextColor(TEXT_SECONDARY),
    ));
    sec.spawn((
        Node {
            width: Val::Percent(100.0), height: Val::Px(14.0),
            ..default()
        },
        BackgroundColor(BAR_BG),
        FuelBarBg,
    ))
    .with_children(|bg| {
        bg.spawn((
            Node {
                width: Val::Percent(0.0), height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(ACCENT),
            FuelBarFill,
        ));
    });
}

fn spawn_section(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    title: &str,
    content: impl FnOnce(&mut bevy::ecs::hierarchy::ChildSpawnerCommands),
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                margin: UiRect::bottom(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(BG_SECTION),
        ))
        .with_children(|sec| {
            sec.spawn((
                Text::new(title),
                tf(10.0),
                TextColor(TEXT_SECONDARY),
                Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() },
            ));
            content(sec);
        });
}
