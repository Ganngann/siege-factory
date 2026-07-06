use crate::economy::building::BuildingRegistry;
use crate::economy::components::{
    AlertText, Building, BuildingPanel, CapacityBarFill, CapacityBarText, ConnectionRowText,
    FarmCropSelectButton, FarmCropText, FarmCultivatorCountText, FarmRecruitButton, FlowInputText,
    FlowOutputText, HpBarFill, HpText, PanelOverlay, PowerStatusText, ProgressBarBg,
    ProgressBarFill, RecipeChangeButton, RecipeNameText, SorterInvertButton, SorterResourceButton,
    StatRowText, StatusText,
};
use crate::economy::resource::ResourceRegistry;
use crate::economy::window::{
    spawn_window, ACCENT, BAR_BG, BG_SECTION, HP_GREEN, TEXT_PRIMARY, TEXT_SECONDARY,
};
use bevy::prelude::*;

pub fn open_panel(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    entity: Entity,
    building: &Building,
    kind: &str,
    resource_registry: &ResourceRegistry,
    _reg: &BuildingRegistry,
    farm_crop_types: Vec<String>,
) {
    // Close existing panel first
    if let Some(e) = panel.root.take() {
        commands.entity(e).try_despawn();
    }
    if let Some(e) = panel.overlay.take() {
        commands.entity(e).try_despawn();
    }
    if let Some(e) = panel.recipe_selector.take() {
        commands.entity(e).try_despawn();
    }
    panel.inspected = None;
    panel.dirty = false;

    let modal_size = Vec2::new(super::MODAL_WIDTH, super::MODAL_HEIGHT);
    let show_recipes = kind == "assembler"
        || kind == "furnace"
        || kind == "blast_furnace"
        || kind == "assembly_crane"
        || kind == "alchemy_lab"
        || kind == "electronics_lab"
        || kind == "foundry"
        || kind == "guild_hall"
        || kind == "enchanting_array"
        || kind == "pumpjack";
    let is_farm = kind == "farm";

    let overlay = commands
        .spawn((
            PanelOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                right: Val::ZERO,
                top: Val::ZERO,
                bottom: Val::ZERO,
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            ZIndex(100),
            Pickable::default(),
        ))
        .id();

    let root = spawn_panel_ui(
        &mut commands,
        modal_size,
        entity,
        building,
        kind,
        show_recipes,
        is_farm,
        &resource_registry,
        farm_crop_types,
    );

    commands.entity(overlay).add_child(root);
    panel.overlay = Some(overlay);
    panel.root = Some(root);
    panel.inspected = Some(entity);
    panel.dirty = true;
}

fn spawn_panel_ui(
    commands: &mut Commands,
    modal_size: Vec2,
    entity: Entity,
    building: &Building,
    kind: &str,
    show_recipes: bool,
    is_farm: bool,
    resource_registry: &ResourceRegistry,
    farm_crop_types: Vec<String>,
) -> Entity {
    let panel_w = modal_size.x;
    let col_w = panel_w * 0.58;
    let right_w = panel_w * 0.38;
    let show_sorter = kind == "sorter";

    let x = (1280.0 - modal_size.x) / 2.0;
    let y = (720.0 - modal_size.y) / 2.0;

    spawn_window(
        commands,
        &format!("{}  #{}", building.name, entity.to_bits() % 1000),
        modal_size.x, modal_size.y, x, y, None,
        |parent| {
            // ── Status bar ──
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
                        TextFont::from_font_size(12.0),
                        TextColor(TEXT_SECONDARY),
                        Node {
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                    ));
                });

            // ── Content row (left | right) ──
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },))
                .with_children(|row| {
                    // ── Left column (Flow + Inventory + Connections) ──
                    row.spawn((Node {
                        width: Val::Px(col_w),
                        flex_direction: FlexDirection::Column,
                        margin: UiRect::right(Val::Px(10.0)),
                        ..default()
                    },))
                        .with_children(|left| {
                            spawn_section(left, "FLOW", |sec| {
                                sec.spawn((
                                    FlowInputText,
                                    Text::new("Inputs: --"),
                                    TextFont::from_font_size(12.0),
                                    TextColor(TEXT_SECONDARY),
                                    Node {
                                        margin: UiRect::bottom(Val::Px(2.0)),
                                        ..default()
                                    },
                                ));
                                sec.spawn((
                                    FlowOutputText,
                                    Text::new("Outputs: --"),
                                    TextFont::from_font_size(12.0),
                                    TextColor(TEXT_SECONDARY),
                                ));
                            });

                            spawn_section(left, "INVENTORY", |sec| {
                                // Capacity bar
                                sec.spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(super::BAR_HEIGHT),
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
                                    TextFont::from_font_size(super::SECTION_FONT_SIZE),
                                    TextColor(TEXT_SECONDARY),
                                    Node {
                                        margin: UiRect::vertical(Val::Px(4.0)),
                                        ..default()
                                    },
                                ));
                                // Building inventory grid (3×2 slots)
                                const S2: f32 = 40.0;
                                const G2: f32 = 3.0;
                                sec.spawn((
                                    crate::economy::components::InventoryGrid { cols: 3, rows: 2, owner: entity },
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
                                            crate::economy::components::InventorySlot { index: i },
                                            Button,
                                            Node {
                                                width: Val::Px(S2),
                                                height: Val::Px(S2),
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
                                            TextFont::from_font_size(9.0),
                                            TextColor(Color::WHITE),
                                        ));
                                    }
                                });
                            });

                            spawn_section(left, "CONNECTIONS", |sec| {
                                sec.spawn((
                                    ConnectionRowText,
                                    Text::new("No connections"),
                                    TextFont::from_font_size(super::SECTION_FONT_SIZE),
                                    TextColor(TEXT_SECONDARY),
                                ));
                            });
                        });

                    // ── Right column (Stats + Settings + HP + Alerts) ──
                    row.spawn((Node {
                        width: Val::Px(right_w),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },))
                        .with_children(|right| {
                            spawn_section(right, "STATS", |sec| {
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
                                        TextFont::from_font_size(super::SECTION_FONT_SIZE),
                                        TextColor(TEXT_SECONDARY),
                                        Node {
                                            margin: UiRect::bottom(Val::Px(1.0)),
                                            ..default()
                                        },
                                    ));
                                }
                            });

                            spawn_section(right, "POWER", |sec| {
                                sec.spawn((
                                    PowerStatusText,
                                    Text::new("Power: --"),
                                    TextFont::from_font_size(super::SECTION_FONT_SIZE),
                                    TextColor(TEXT_SECONDARY),
                                ));
                            });

                            if show_recipes {
                                spawn_section(right, "SETTINGS", |sec| {
                                    sec.spawn((
                                        RecipeNameText,
                                        Text::new("Recipe: --"),
                                        TextFont::from_font_size(12.0),
                                        TextColor(TEXT_PRIMARY),
                                        Node {
                                            margin: UiRect::bottom(Val::Px(4.0)),
                                            ..default()
                                        },
                                    ));
                                    sec.spawn((
                                        RecipeChangeButton,
                                        Button,
                                        Node {
                                            width: Val::Px(120.0),
                                            height: Val::Px(24.0),
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.18, 0.25, 0.40)),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("[Change Recipe]"),
                                            TextFont::from_font_size(super::SECTION_FONT_SIZE),
                                            TextColor(ACCENT),
                                        ));
                                    });
                                });
                            }

                            if is_farm {
                                spawn_section(right, "FARM", |sec| {
                                    sec.spawn((
                                        FarmCropText,
                                        Text::new("Crops:  --"),
                                        TextFont::from_font_size(12.0),
                                        TextColor(TEXT_PRIMARY),
                                        Node {
                                            margin: UiRect::bottom(Val::Px(4.0)),
                                            ..default()
                                        },
                                    ));

                                    // Crop type selection buttons
                                    for crop_type in &farm_crop_types {
                                        sec.spawn((
                                            FarmCropSelectButton {
                                                crop_type: crop_type.clone(),
                                            },
                                            Button,
                                            Node {
                                                width: Val::Px(120.0),
                                                height: Val::Px(22.0),
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::Center,
                                                margin: UiRect::vertical(Val::Px(1.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.18, 0.35, 0.18)),
                                        ))
                                        .with_children(
                                            |btn| {
                                                btn.spawn((
                                                    Text::new(crop_type),
                                                    TextFont::from_font_size(super::SECTION_FONT_SIZE),
                                                    TextColor(TEXT_SECONDARY),
                                                ));
                                            },
                                        );
                                    }

                                    sec.spawn((
                                        FarmCultivatorCountText,
                                        Text::new("Cultivators:  --"),
                                        TextFont::from_font_size(12.0),
                                        TextColor(TEXT_SECONDARY),
                                        Node {
                                            margin: UiRect::vertical(Val::Px(4.0)),
                                            ..default()
                                        },
                                    ));
                                    sec.spawn((
                                        FarmRecruitButton,
                                        Button,
                                        Node {
                                            width: Val::Px(160.0),
                                            height: Val::Px(super::CLOSE_BUTTON_SIZE),
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.25, 0.55, 0.25)),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("[Recruit Cultivator]  8 ore"),
                                            TextFont::from_font_size(12.0),
                                            TextColor(TEXT_PRIMARY),
                                        ));
                                    });
                                });
                            }

                            if show_sorter {
                                spawn_section(right, "FILTER", |sec| {
                                    sec.spawn((
                                        SorterInvertButton,
                                        Button,
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(super::CLOSE_BUTTON_SIZE),
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
                                            TextFont::from_font_size(12.0),
                                            TextColor(TEXT_PRIMARY),
                                        ));
                                    });

                                    let mut resources: Vec<String> = resource_registry
                                        .resources
                                        .keys()
                                        .map(|k| k.clone())
                                        .collect();
                                    resources.sort();
                                    for res in &resources {
                                        sec.spawn((
                                            SorterResourceButton {
                                                resource: crate::economy::resource::ResourceId(
                                                    res.clone(),
                                                ),
                                            },
                                            Button,
                                            Node {
                                                width: Val::Percent(100.0),
                                                height: Val::Px(22.0),
                                                align_items: AlignItems::Center,
                                                padding: UiRect::horizontal(Val::Px(8.0)),
                                                margin: UiRect::vertical(Val::Px(1.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.15, 0.15, 0.22)),
                                        ))
                                        .with_children(
                                            |btn| {
                                                btn.spawn((
                                                    Text::new(res),
                                                    TextFont::from_font_size(super::SECTION_FONT_SIZE),
                                                    TextColor(TEXT_SECONDARY),
                                                ));
                                            },
                                        );
                                    }
                                });
                            }

                            spawn_section(right, "HP", |sec| {
                                sec.spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(super::BAR_HEIGHT),
                                        ..default()
                                    },
                                    BackgroundColor(BAR_BG),
                                ))
                                .with_children(|bar| {
                                    bar.spawn((
                                        HpBarFill,
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        BackgroundColor(HP_GREEN),
                                    ));
                                });
                                sec.spawn((
                                    HpText,
                                    Text::new("HP: --/--"),
                                    TextFont::from_font_size(super::SECTION_FONT_SIZE),
                                    TextColor(TEXT_SECONDARY),
                                    Node {
                                        margin: UiRect::top(Val::Px(4.0)),
                                        ..default()
                                    },
                                ));
                            });

                            spawn_section(right, "ALERTS", |sec| {
                                sec.spawn((
                                    AlertText,
                                    Text::new("No alerts"),
                                    TextFont::from_font_size(super::SECTION_FONT_SIZE),
                                    TextColor(TEXT_SECONDARY),
                                ));
                            });
                        });
                        });
        })
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
                TextFont::from_font_size(10.0),
                TextColor(TEXT_SECONDARY),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));
            content(sec);
        });
}
