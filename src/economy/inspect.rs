use crate::agriculture::components::{CropRegistry, Cultivator, Farm};
use crate::core::input::KeyBindings;
use crate::core::toast::ToastQueue;
use crate::economy::belt::BeltSlots;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::HQ;
use crate::economy::components::{
    Active, ActiveToggleButton, AlertText, Assembler, BuildMode, Building, BuildingPanel,
    BuildingTitleText, CapacityBarFill, CapacityBarText, CloseButton, ConnectionRowText,
    DeconstructMode, DragHandle, FarmCropSelectButton, FarmCropText, FarmCultivatorCountText,
    FarmRecruitButton, FlowInputText, FlowOutputText, HpBarFill, HpText, PanelModal, PanelOverlay,
    ProgressBarBg, ProgressBarFill, RecipeCategoryLabel, RecipeChangeButton, RecipeNameText,
    RecipeSelectorItem, RecipeSelectorRoot, ResourceDeposit, Sorter, SorterInvertButton,
    SorterResourceButton, StatRowText, StatusText, UiIsBlocking,
};
use crate::economy::recipe::RecipeRegistry;
use crate::economy::resource::ResourceId;
use crate::economy::resource::{Inventory, ResourceRegistry};
use crate::economy::spatial::SpatialRegistry;
use crate::economy::unit_config::UnitConfig;
use crate::enemy::components::Health;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use bevy::prelude::*;

// ── Colors ──

const BG_SECTION: Color = Color::srgb(0.10, 0.10, 0.18);
const BG_MODAL: Color = Color::srgba(0.08, 0.08, 0.16, 0.97);
const ACCENT: Color = Color::srgb(0.30, 0.55, 1.00);
const TEXT_PRIMARY: Color = Color::srgb(0.90, 0.90, 1.00);
const TEXT_SECONDARY: Color = Color::srgb(0.60, 0.60, 0.75);
const TEXT_GREEN: Color = Color::srgb(0.40, 0.85, 0.40);
const TEXT_YELLOW: Color = Color::srgb(0.85, 0.85, 0.35);
const BTN_CLOSE: Color = Color::srgb(0.50, 0.12, 0.12);
const BTN_ACTIVE: Color = Color::srgb(0.15, 0.45, 0.15);
const BTN_INACTIVE: Color = Color::srgb(0.30, 0.15, 0.15);
const HP_GREEN: Color = Color::srgb(0.20, 0.65, 0.20);
const BAR_BG: Color = Color::srgb(0.15, 0.15, 0.22);
const SEPARATOR: Color = Color::srgb(0.20, 0.20, 0.30);

// ── Open / close panel ──

fn close_panel_impl(commands: &mut Commands, panel: &mut BuildingPanel) {
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
}

pub fn close_panel(mut commands: Commands, mut panel: ResMut<BuildingPanel>) {
    close_panel_impl(&mut commands, &mut panel);
}

fn open_panel(
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

    let modal_size = Vec2::new(800.0, 560.0);
    let show_recipes = kind == "assembler" || kind == "furnace";
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

    commands
        .spawn((
            PanelModal,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px((1280.0 - modal_size.x) / 2.0),
                top: Val::Px((720.0 - modal_size.y) / 2.0),
                flex_direction: FlexDirection::Column,
                width: Val::Px(modal_size.x),
                height: Val::Px(modal_size.y),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(BG_MODAL),
            Outline {
                width: Val::Px(1.0),
                offset: Val::ZERO,
                color: Color::srgb(0.30, 0.30, 0.45),
            },
            ZIndex(101),
        ))
        .with_children(|parent| {
            // ── Header ──
            parent
                .spawn((
                    DragHandle,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(40.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::horizontal(Val::Px(14.0)),
                        border: UiRect::bottom(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(BG_SECTION),
                    BorderColor {
                        top: SEPARATOR,
                        bottom: SEPARATOR,
                        left: SEPARATOR,
                        right: SEPARATOR,
                    },
                ))
                .with_children(|header| {
                    header.spawn((
                        BuildingTitleText,
                        Text::new(format!("{}  #{}", building.name, entity.to_bits() % 1000)),
                        TextFont::from_font_size(18.0),
                        TextColor(TEXT_PRIMARY),
                    ));
                    header
                        .spawn((
                            ActiveToggleButton,
                            Button,
                            Node {
                                width: Val::Px(60.0),
                                height: Val::Px(26.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                margin: UiRect::right(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(BTN_ACTIVE),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("[ON]"),
                                TextFont::from_font_size(12.0),
                                TextColor(TEXT_GREEN),
                            ));
                        });
                    header
                        .spawn((
                            CloseButton,
                            Button,
                            Node {
                                width: Val::Px(28.0),
                                height: Val::Px(28.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_CLOSE),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("X"),
                                TextFont::from_font_size(16.0),
                                TextColor(Color::WHITE),
                            ));
                        });
                });

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
                                    TextFont::from_font_size(11.0),
                                    TextColor(TEXT_SECONDARY),
                                    Node {
                                        margin: UiRect::vertical(Val::Px(4.0)),
                                        ..default()
                                    },
                                ));
                            });

                            spawn_section(left, "CONNECTIONS", |sec| {
                                sec.spawn((
                                    ConnectionRowText,
                                    Text::new("No connections"),
                                    TextFont::from_font_size(11.0),
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
                                        TextFont::from_font_size(11.0),
                                        TextColor(TEXT_SECONDARY),
                                        Node {
                                            margin: UiRect::bottom(Val::Px(1.0)),
                                            ..default()
                                        },
                                    ));
                                }
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
                                            TextFont::from_font_size(11.0),
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
                                                    TextFont::from_font_size(11.0),
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
                                            height: Val::Px(26.0),
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
                                            height: Val::Px(26.0),
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
                                                    TextFont::from_font_size(11.0),
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
                                        height: Val::Px(12.0),
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
                                    TextFont::from_font_size(11.0),
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
                                    TextFont::from_font_size(11.0),
                                    TextColor(TEXT_SECONDARY),
                                ));
                            });
                        });
                });
        })
        .id()
}

fn spawn_deposit_panel(
    commands: &mut Commands,
    panel: &mut BuildingPanel,
    entity: Entity,
    deposit: &ResourceDeposit,
    resource_registry: &ResourceRegistry,
) {
    let resource_name = resource_registry
        .get_opt(&deposit.resource)
        .map_or(deposit.resource.as_str(), |r| &r.name);

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

    let root = commands
        .spawn((
            PanelModal,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px((1280.0 - 400.0) / 2.0),
                top: Val::Px((720.0 - 200.0) / 2.0),
                flex_direction: FlexDirection::Column,
                width: Val::Px(400.0),
                height: Val::Px(200.0),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(BG_MODAL),
            Outline {
                width: Val::Px(1.0),
                offset: Val::ZERO,
                color: Color::srgb(0.30, 0.30, 0.45),
            },
            ZIndex(101),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    DragHandle,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(40.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::horizontal(Val::Px(14.0)),
                        border: UiRect::bottom(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(BG_SECTION),
                    BorderColor {
                        top: SEPARATOR,
                        bottom: SEPARATOR,
                        left: SEPARATOR,
                        right: SEPARATOR,
                    },
                ))
                .with_children(|header| {
                    header.spawn((
                        BuildingTitleText,
                        Text::new(format!("Resource Deposit: {}", resource_name)),
                        TextFont::from_font_size(16.0),
                        TextColor(TEXT_PRIMARY),
                    ));
                    header
                        .spawn((
                            CloseButton,
                            Button,
                            Node {
                                width: Val::Px(28.0),
                                height: Val::Px(28.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_CLOSE),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("X"),
                                TextFont::from_font_size(16.0),
                                TextColor(Color::WHITE),
                            ));
                        });
                });

            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },))
                .with_children(|body| {
                    body.spawn((
                        Text::new(format!("Resource: {}", resource_name)),
                        TextFont::from_font_size(14.0),
                        TextColor(TEXT_PRIMARY),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ));
                    body.spawn((
                        Text::new(format!("Remaining: {}", deposit.amount)),
                        TextFont::from_font_size(14.0),
                        TextColor(TEXT_GREEN),
                    ));
                });
        })
        .id();

    commands.entity(overlay).add_child(root);
    panel.overlay = Some(overlay);
    panel.root = Some(root);
    panel.inspected = Some(entity);
    panel.dirty = true;
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

// ── Update panel content (separate systems for each section) ──

pub fn update_panel_header(
    panel: Res<BuildingPanel>,
    building_query: Query<(&Building, Option<&Active>)>,
    mut title_text: Query<&mut Text, (With<BuildingTitleText>, Without<ActiveToggleButton>)>,
    mut toggle_btn: Query<
        (&mut BackgroundColor, &mut Text),
        (With<ActiveToggleButton>, Without<BuildingTitleText>),
    >,
) {
    let Some(inspected) = panel.inspected else {
        return;
    };
    if panel.root.is_none() {
        return;
    }
    let Ok((building, active)) = building_query.get(inspected) else {
        return;
    };
    let is_active = active.and_then(|a| Some(a.0)).unwrap_or(true);

    if let Ok(mut t) = title_text.single_mut() {
        t.0 = format!("{}  #{}", building.name, inspected.to_bits() % 1000);
    }
    if let Ok((mut bg, mut text)) = toggle_btn.single_mut() {
        if is_active {
            *bg = BackgroundColor(BTN_ACTIVE);
            text.0 = "[ON]".to_string();
        } else {
            *bg = BackgroundColor(BTN_INACTIVE);
            text.0 = "[OFF]".to_string();
        }
    }
}

pub fn update_panel_production(
    panel: Res<BuildingPanel>,
    building_query: Query<(&Building, Option<&Assembler>, Option<&Active>)>,
    recipes: Res<RecipeRegistry>,
    resource_registry: Res<ResourceRegistry>,
    mut progress_fill: Query<&mut Node, With<ProgressBarFill>>,
    mut status_text: Query<
        &mut Text,
        (
            With<StatusText>,
            Without<FlowInputText>,
            Without<FlowOutputText>,
        ),
    >,
    mut flow_input: Query<
        &mut Text,
        (
            With<FlowInputText>,
            Without<StatusText>,
            Without<FlowOutputText>,
        ),
    >,
    mut flow_output: Query<
        &mut Text,
        (
            With<FlowOutputText>,
            Without<StatusText>,
            Without<FlowInputText>,
        ),
    >,
) {
    let Some(inspected) = panel.inspected else {
        return;
    };
    let Ok((_building, assembler, active)) = building_query.get(inspected) else {
        return;
    };
    let is_active = active.and_then(|a| Some(a.0)).unwrap_or(true);

    let progress_pct: f32;
    let status_str: String;

    if let Some(asm) = assembler {
        let is_mining = asm.recipe_id.starts_with("mine_");
        let display_name = if is_mining {
            let resource = &asm.recipe_id[5..];
            resource_registry
                .get_opt(resource)
                .map_or(resource.to_string(), |r| r.name.clone())
        } else {
            asm.recipe_id.clone()
        };

        if let Some(def) = recipes.get(&asm.recipe_id) {
            let pct = if asm.production_timer >= def.time_sec {
                100.0
            } else {
                (asm.production_timer / def.time_sec * 100.0).min(100.0)
            };
            progress_pct = pct;
            if is_active && asm.production_timer > 0.0 {
                status_str = if is_mining {
                    format!(
                        "Mining: {}  -  {:.1}s / {:.1}s",
                        display_name, asm.production_timer, def.time_sec
                    )
                } else {
                    format!(
                        "Producing: {}  -  {:.1}s / {:.1}s",
                        display_name, asm.production_timer, def.time_sec
                    )
                };
            } else if !is_active {
                status_str = "Paused".to_string();
            } else {
                status_str = format!("Ready: {}", display_name);
            }

            if let Ok(mut inp) = flow_input.single_mut() {
                if is_mining || def.input.is_empty() {
                    inp.0 = "Inputs:  (raw material)".to_string();
                } else {
                    let parts: Vec<String> = def
                        .input
                        .iter()
                        .map(|(rid, amt)| {
                            let name = resource_registry
                                .get_opt(&rid.0)
                                .map_or(rid.0.as_str(), |r| &r.name);
                            format!("{} x{}", name, amt)
                        })
                        .collect();
                    inp.0 = format!("Inputs:  {}", parts.join("  "));
                }
            }
            if let Ok(mut out) = flow_output.single_mut() {
                let parts: Vec<String> = def
                    .output
                    .iter()
                    .map(|(rid, amt)| {
                        let name = resource_registry
                            .get_opt(&rid.0)
                            .map_or(rid.0.as_str(), |r| &r.name);
                        format!("{}  \u{d7}{}", name, amt)
                    })
                    .collect();
                out.0 = format!("Outputs:  {}", parts.join("  "));
            }
        } else {
            progress_pct = 0.0;
            status_str = format!("Active: {}", display_name);
        }
    } else {
        progress_pct = 0.0;
        status_str = "Idle".to_string();
        if let Ok(mut inp) = flow_input.single_mut() {
            inp.0 = "Inputs:  --".to_string();
        }
        if let Ok(mut out) = flow_output.single_mut() {
            out.0 = "Outputs:  --".to_string();
        }
    }

    if let Ok(mut fill) = progress_fill.single_mut() {
        fill.width = Val::Percent(progress_pct);
    }
    if let Ok(mut st) = status_text.single_mut() {
        st.0 = status_str;
    }
}

pub fn update_panel_inventory(
    panel: Res<BuildingPanel>,
    inventory_query: Query<Option<&Inventory>>,
    resource_registry: Res<ResourceRegistry>,
    mut cap_bar: Query<&mut Node, With<CapacityBarFill>>,
    mut cap_text: Query<&mut Text, With<CapacityBarText>>,
) {
    let Some(inspected) = panel.inspected else {
        return;
    };
    let Ok(inventory) = inventory_query.get(inspected) else {
        return;
    };
    let Some(inv) = inventory else { return };

    if let Ok(mut cap) = cap_bar.single_mut() {
        let pct = if inv.capacity > 0 {
            (inv.total() as f32 / inv.capacity as f32 * 100.0).min(100.0)
        } else {
            0.0
        };
        cap.width = Val::Percent(pct);
    }
    if let Ok(mut ct) = cap_text.single_mut() {
        if inv.capacity > 0 {
            ct.0 = format!("Capacity:  {}/{}", inv.total(), inv.capacity);
        } else if inv.total() > 0 {
            let mut lines: Vec<String> = Vec::new();
            let mut sorted: Vec<_> = inv.resources.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));
            for (rid, amount) in sorted.iter().take(5) {
                let name = resource_registry
                    .get_opt(&rid.0)
                    .map_or(rid.0.as_str(), |r| &r.name);
                lines.push(format!("{}: {}", name, amount));
            }
            if inv.resources.len() > 5 {
                lines.push(format!("... +{} more", inv.resources.len() - 5));
            }
            ct.0 = lines.join("  |  ");
        } else {
            ct.0 = format!("Items:  {}", inv.total());
        }
    }
}

pub fn update_panel_connections(
    panel: Res<BuildingPanel>,
    belt_query: Query<Option<&BeltSlots>>,
    mut conn_text: Query<&mut Text, With<ConnectionRowText>>,
) {
    let Some(inspected) = panel.inspected else {
        return;
    };
    let Ok(belt) = belt_query.get(inspected) else {
        return;
    };

    if let Some(bs) = belt {
        if let Ok(mut ct) = conn_text.single_mut() {
            let occupied = bs.items.iter().filter(|s| s.is_some()).count();
            ct.0 = format!(
                "Items in transit:  {}/{}  |  {:?}",
                occupied,
                bs.items.len(),
                bs.direction
            );
        }
    } else {
        if let Ok(mut ct) = conn_text.single_mut() {
            ct.0 = "No connections".to_string();
        }
    }
}

pub fn update_panel_stats(
    panel: Res<BuildingPanel>,
    assembler_query: Query<Option<&Assembler>>,
    mut stat_rows: Query<&mut Text, (With<StatRowText>, Without<RecipeNameText>)>,
    mut recipe_name: Query<&mut Text, (With<RecipeNameText>, Without<StatRowText>)>,
) {
    let Some(inspected) = panel.inspected else {
        return;
    };
    let Ok(assembler) = assembler_query.get(inspected) else {
        return;
    };

    for (i, mut text) in stat_rows.iter_mut().enumerate() {
        if i > 4 {
            break;
        }
        let stats = [
            format!("Produced/min:  --"),
            format!("Consumed/min:  --"),
            format!("Uptime:        --"),
            format!("Efficiency:    --"),
            format!("Total output:  0"),
        ];
        text.0 = stats[i].clone();
    }

    if let Some(asm) = assembler {
        if let Ok(mut rn) = recipe_name.single_mut() {
            let display = if asm.recipe_id.starts_with("mine_") {
                let resource = &asm.recipe_id[5..];
                format!("Mining: {}", resource)
            } else {
                format!("Recipe:  {}", asm.recipe_id)
            };
            rn.0 = display;
        }
    }
}

pub fn update_panel_hp(
    panel: Res<BuildingPanel>,
    health_query: Query<Option<&Health>>,
    mut hp_fill: Query<&mut Node, With<HpBarFill>>,
    mut hp_text: Query<&mut Text, With<HpText>>,
) {
    let Some(inspected) = panel.inspected else {
        return;
    };
    let Ok(health) = health_query.get(inspected) else {
        return;
    };

    if let Some(h) = health {
        if let Ok(mut fill) = hp_fill.single_mut() {
            let pct = if h.max > 0 {
                (h.current as f32 / h.max as f32 * 100.0).min(100.0)
            } else {
                0.0
            };
            fill.width = Val::Percent(pct);
        }
        if let Ok(mut ht) = hp_text.single_mut() {
            ht.0 = format!("HP:  {}/{}", h.current, h.max);
        }
    }
}

pub fn update_panel_alerts(
    panel: Res<BuildingPanel>,
    active_query: Query<Option<&Active>>,
    mut alert_text: Query<&mut Text, With<AlertText>>,
) {
    let Some(inspected) = panel.inspected else {
        return;
    };
    let Ok(active) = active_query.get(inspected) else {
        return;
    };

    let is_active = active.and_then(|a| Some(a.0)).unwrap_or(true);
    let mut alerts: Vec<String> = Vec::new();
    if !is_active {
        alerts.push("[!] Building paused".to_string());
    }
    if let Ok(mut at) = alert_text.single_mut() {
        if alerts.is_empty() {
            at.0 = "No alerts".to_string();
        } else {
            at.0 = alerts.join("\n");
        }
    }
}

// ── Click detection ──

pub fn building_inspect_click(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    build_mode: Res<BuildMode>,
    deconstruct: Res<DeconstructMode>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    cfg: Res<MapConfig>,
    spatial: Res<SpatialRegistry>,
    building_query: Query<&Building>,
    deposit_query: Query<(Entity, &ResourceDeposit, &TilePosition)>,
    resource_registry: Res<ResourceRegistry>,
    reg: Res<BuildingRegistry>,
    ui_blocking: Res<UiIsBlocking>,
) {
    if ui_blocking.0 {
        return;
    }
    if build_mode.0.is_some() || deconstruct.0 {
        return;
    }
    if !bindings.just_pressed("place", &keys, &buttons) {
        return;
    }

    let tile_size = cfg.tile_size;
    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_transform)) = camera.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else {
        return;
    };

    let tile_x = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let tile_y = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

    // Check deposits first (they are NOT in SpatialRegistry)
    if let Some((deposit_entity, deposit, _)) = deposit_query
        .iter()
        .find(|(_, _, pos)| pos.x == tile_x && pos.y == tile_y)
    {
        if panel.inspected == Some(deposit_entity) {
            close_panel(commands, panel);
            return;
        }
        spawn_deposit_panel(
            &mut commands,
            &mut *panel,
            deposit_entity,
            deposit,
            &resource_registry,
        );
        return;
    }

    let Some(entity) = spatial.at(tile_x, tile_y) else {
        return;
    };
    if panel.inspected == Some(entity) {
        close_panel(commands, panel);
        return;
    }

    if let Ok(building) = building_query.get(entity) {
        let farm_crop_types = if building.kind == "farm" {
            vec!["wheat".to_string(), "wood".to_string()]
        } else {
            Vec::new()
        };
        open_panel(
            commands,
            panel,
            entity,
            building,
            &building.kind,
            &resource_registry,
            &reg,
            farm_crop_types,
        );
    }
}

// ── Overlay click to close ──

pub fn overlay_click_system(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    modal_query: Query<(&Node, &GlobalTransform), (With<PanelModal>, Without<PanelOverlay>)>,
) {
    if panel.overlay.is_none() {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    // If click inside the modal body → let the modal's own buttons handle it
    if let Ok((_node, transform)) = modal_query.single() {
        let center = transform.translation().truncate();
        let modal_rect = Rect::from_center_size(center, Vec2::new(800.0, 560.0));
        if modal_rect.contains(cursor) {
            return;
        }
    } else {
        return;
    }

    // Click is outside the modal → close
    if panel.recipe_selector.is_some() {
        if let Some(e) = panel.recipe_selector.take() {
            commands.entity(e).try_despawn();
        }
    } else {
        close_panel(commands, panel);
    }
}

// ── Close button ──

pub fn close_button_system(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    query: Query<&Interaction, (Changed<Interaction>, With<CloseButton>)>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if panel.recipe_selector.is_some() {
            if let Some(e) = panel.recipe_selector.take() {
                commands.entity(e).try_despawn();
            }
        } else {
            close_panel(commands, panel);
        }
        return;
    }
}

// ── Escape to close ──

pub fn close_popup_on_escape(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
) {
    if !keys.just_pressed(bindings.key("cancel")) {
        return;
    }
    if panel.recipe_selector.is_some() {
        if let Some(e) = panel.recipe_selector.take() {
            commands.entity(e).try_despawn();
        }
    } else if panel.overlay.is_some() {
        close_panel(commands, panel);
    }
}

// ── Active toggle ──

pub fn active_toggle_system(
    panel: ResMut<BuildingPanel>,
    query: Query<&Interaction, (Changed<Interaction>, With<ActiveToggleButton>)>,
    mut active_query: Query<&mut Active>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(inspected) = panel.inspected else {
            continue;
        };
        if let Ok(mut active) = active_query.get_mut(inspected) {
            active.0 = !active.0;
        }
    }
}

// ── Recipe change button → open selector ──

pub fn recipe_change_system(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    query: Query<&Interaction, (Changed<Interaction>, With<RecipeChangeButton>)>,
    building_query: Query<&Building>,
    assembler_query: Query<&Assembler>,
    inventory_query: Query<Option<&Inventory>>,
    recipes: Res<RecipeRegistry>,
    resource_registry: Res<ResourceRegistry>,
    reg: Res<BuildingRegistry>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(inspected) = panel.inspected else {
            continue;
        };
        let Ok(building) = building_query.get(inspected) else {
            continue;
        };
        let Ok(asm) = assembler_query.get(inspected) else {
            continue;
        };

        if let Some(e) = panel.recipe_selector.take() {
            commands.entity(e).try_despawn();
        }

        let categories = reg
            .get(&building.kind)
            .map(|def| def.recipe_categories.clone())
            .unwrap_or_default();

        // Use the building's own inventory to determine which recipes are craftable
        let building_inv = inventory_query.get(inspected).ok().and_then(|o| o);

        let sel = spawn_recipe_selector(
            &mut commands,
            &asm.recipe_id,
            &categories,
            &recipes,
            &resource_registry,
            building_inv,
        );
        if let Some(root) = panel.root {
            commands.entity(root).add_child(sel);
        }
        panel.recipe_selector = Some(sel);
    }
}

fn spawn_recipe_selector(
    commands: &mut Commands,
    current_id: &str,
    categories: &[String],
    recipes: &RecipeRegistry,
    resource_registry: &ResourceRegistry,
    building_inv: Option<&Inventory>,
) -> Entity {
    commands
        .spawn((
            RecipeSelectorRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(420.0),
                top: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                width: Val::Px(360.0),
                height: Val::Px(300.0),
                padding: UiRect::all(Val::Px(10.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.10, 0.20, 0.98)),
            Outline {
                width: Val::Px(1.0),
                offset: Val::ZERO,
                color: Color::srgb(0.40, 0.40, 0.55),
            },
            ZIndex(102),
        ))
        .with_children(|parent| {
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },))
                .with_children(|title| {
                    title.spawn((
                        Text::new("Select Recipe"),
                        TextFont::from_font_size(14.0),
                        TextColor(TEXT_PRIMARY),
                    ));
                    title
                        .spawn((
                            CloseButton,
                            Button,
                            Node {
                                width: Val::Px(20.0),
                                height: Val::Px(20.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_CLOSE),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("X"),
                                TextFont::from_font_size(12.0),
                                TextColor(Color::WHITE),
                            ));
                        });
                });

            let mut seen_categories: Vec<(String, Vec<&crate::economy::recipe::RecipeDef>)> =
                Vec::new();
            for cat in categories {
                let mut cat_recipes: Vec<&crate::economy::recipe::RecipeDef> = recipes
                    .recipes
                    .values()
                    .filter(|r| r.category == *cat)
                    .collect();
                cat_recipes.sort_by(|a, b| a.id.cmp(&b.id));
                if !cat_recipes.is_empty() {
                    seen_categories.push((cat.clone(), cat_recipes));
                }
            }

            for (cat_name, cat_recipes) in &seen_categories {
                parent.spawn((
                    RecipeCategoryLabel,
                    Text::new(format!("-- {} --", cat_name.to_uppercase())),
                    TextFont::from_font_size(11.0),
                    TextColor(TEXT_YELLOW),
                    Node {
                        margin: UiRect::vertical(Val::Px(4.0)),
                        ..default()
                    },
                ));

                for recipe in cat_recipes {
                    let is_active = recipe.id == current_id;

                    let can_craft = if recipe.input.is_empty() {
                        recipe.input.is_empty()
                    } else {
                        building_inv.map_or(false, |inv| {
                            recipe.input.iter().all(|(rid, amt)| inv.get(rid) >= *amt)
                        })
                    };

                    let bg = if is_active {
                        Color::srgb(0.20, 0.50, 0.20)
                    } else if can_craft {
                        Color::srgb(0.18, 0.35, 0.18)
                    } else {
                        Color::srgb(0.12, 0.12, 0.20)
                    };
                    let border_color = if can_craft && !is_active {
                        Color::srgb(0.30, 0.70, 0.30)
                    } else {
                        Color::srgb(0.20, 0.20, 0.30)
                    };
                    let prefix = if is_active {
                        "> "
                    } else if can_craft {
                        "[x] "
                    } else {
                        "    "
                    };

                    let input_str: String = recipe
                        .input
                        .iter()
                        .map(|(rid, amt)| {
                            let name = resource_registry
                                .get_opt(&rid.0)
                                .map_or(rid.0.as_str(), |r| &r.name);
                            format!("{} x{}", name, amt)
                        })
                        .collect::<Vec<_>>()
                        .join(" + ");
                    let output_str: String = recipe
                        .output
                        .iter()
                        .map(|(rid, amt)| {
                            let name = resource_registry
                                .get_opt(&rid.0)
                                .map_or(rid.0.as_str(), |r| &r.name);
                            format!("{} x{}", name, amt)
                        })
                        .collect::<Vec<_>>()
                        .join(" + ");

                    parent
                        .spawn((
                            RecipeSelectorItem {
                                recipe_id: recipe.id.clone(),
                            },
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(34.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(6.0)),
                                margin: UiRect::vertical(Val::Px(1.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(bg),
                            BorderColor::all(border_color),
                        ))
                        .with_children(|btn| {
                            let recipe_name = resource_registry
                                .get_opt(&recipe.id)
                                .map_or(recipe.id.as_str(), |r| &r.name);
                            btn.spawn((
                                Text::new(format!("{}{}", prefix, recipe_name)),
                                TextFont::from_font_size(12.0),
                                TextColor(if is_active {
                                    TEXT_GREEN
                                } else if can_craft {
                                    TEXT_PRIMARY
                                } else {
                                    TEXT_SECONDARY
                                }),
                            ));
                            btn.spawn((
                                Text::new(format!(
                                    "    {}  ->  {}  |  {:.1}s",
                                    input_str, output_str, recipe.time_sec
                                )),
                                TextFont::from_font_size(10.0),
                                TextColor(TEXT_SECONDARY),
                            ));
                        });
                }
            }
        })
        .id()
}

// ── Recipe selector item click ──

pub fn recipe_selector_click(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    query: Query<(&Interaction, &RecipeSelectorItem), Changed<Interaction>>,
    mut assembler_query: Query<&mut Assembler>,
) {
    for (interaction, item) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(inspected) = panel.inspected else {
            continue;
        };
        if let Ok(mut asm) = assembler_query.get_mut(inspected) {
            asm.recipe_id = item.recipe_id.clone();
            panel.dirty = true;
        }
        if let Some(e) = panel.recipe_selector.take() {
            commands.entity(e).try_despawn();
        }
        return;
    }
}

// ── Sorter resource button click ──

pub fn sorter_resource_click_system(
    mut panel: ResMut<BuildingPanel>,
    query: Query<(&Interaction, &SorterResourceButton), Changed<Interaction>>,
    mut sorter_query: Query<&mut Sorter>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    for (interaction, btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(inspected) = panel.inspected else {
            continue;
        };
        if let Ok(mut sorter) = sorter_query.get_mut(inspected) {
            sorter.filter = btn.resource.clone();
            toast_queue
                .0
                .push(format!("Sorter filter: {}", btn.resource.display_name()));
            panel.dirty = true;
        }
    }
}

// ── Sorter invert button click ──

pub fn sorter_invert_click_system(
    mut panel: ResMut<BuildingPanel>,
    query: Query<&Interaction, (Changed<Interaction>, With<SorterInvertButton>)>,
    mut sorter_query: Query<&mut Sorter>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(inspected) = panel.inspected else {
            continue;
        };
        if let Ok(mut sorter) = sorter_query.get_mut(inspected) {
            sorter.inverted = !sorter.inverted;
            let mode = if sorter.inverted {
                "inverted"
            } else {
                "normal"
            };
            toast_queue.0.push(format!("Sorter: {}", mode));
            panel.dirty = true;
        }
    }
}

// ── Draggable panels ──

#[derive(Resource, Default)]
pub struct PanelDrag {
    pub dragging: bool,
    pub cursor_start: Vec2,
    pub panel_start_left: f32,
    pub panel_start_top: f32,
    pub frame_delay: u32,
}

pub fn drag_panel_system(
    mut drag: ResMut<PanelDrag>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut panel_query: Query<&mut Node, With<PanelModal>>,
) {
    if panel_query.is_empty() {
        *drag = PanelDrag::default();
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    if drag.dragging {
        if buttons.just_released(MouseButton::Left) {
            drag.dragging = false;
        } else if let Ok(mut node) = panel_query.single_mut() {
            let delta = cursor - drag.cursor_start;
            node.left = Val::Px(drag.panel_start_left + delta.x);
            node.top = Val::Px(drag.panel_start_top + delta.y);
        }
        return;
    }

    // Delay drag detection for 2 frames after panel opens
    // to avoid catching the mouse click that triggered panel spawn
    if drag.frame_delay < 2 {
        drag.frame_delay += 1;
        return;
    }

    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(node) = panel_query.single() else {
        return;
    };

    let panel_left = match node.left {
        Val::Px(v) => v,
        _ => 0.0,
    };
    let panel_top = match node.top {
        Val::Px(v) => v,
        _ => 0.0,
    };
    let panel_w = match node.width {
        Val::Px(v) => v,
        _ => 800.0,
    };

    let header_rect = Rect::new(
        panel_left,
        panel_top,
        panel_left + panel_w,
        panel_top + 40.0,
    );
    if header_rect.contains(cursor) {
        drag.dragging = true;
        drag.cursor_start = cursor;
        drag.panel_start_left = panel_left;
        drag.panel_start_top = panel_top;
    }
}

// ── Farm crop select button ──

pub fn farm_crop_select_system(
    query: Query<(&Interaction, &FarmCropSelectButton), Changed<Interaction>>,
    mut farm_query: Query<&mut Farm>,
    panel: Res<BuildingPanel>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    for (interaction, btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(inspected) = panel.inspected else {
            continue;
        };
        if let Ok(mut farm) = farm_query.get_mut(inspected) {
            let idx = farm.crop_types.iter().position(|c| c == &btn.crop_type);
            if let Some(i) = idx {
                farm.crop_index = i;
                toast_queue.0.push(format!("Crop: {}", btn.crop_type));
            }
        }
    }
}

// ── Farm recruit button ──

pub fn farm_recruit_system(
    mut commands: Commands,
    panel: ResMut<BuildingPanel>,
    query: Query<&Interaction, (Changed<Interaction>, With<FarmRecruitButton>)>,
    farm_query: Query<&Farm>,
    farm_tf_query: Query<&Transform, With<Farm>>,
    unit_cfg: Res<UnitConfig>,
    mut hq_inv_query: Query<&mut Inventory, With<HQ>>,
    cfg: Res<MapConfig>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(inspected) = panel.inspected else {
            continue;
        };
        if farm_query.get(inspected).is_err() {
            continue;
        }

        let Some(def) = unit_cfg.get("cultivator") else {
            continue;
        };
        let cost_ore = def
            .cost
            .iter()
            .find(|c| c.resource.0 == "ore")
            .map(|c| c.amount)
            .unwrap_or(8);
        let mut hq_inv = match hq_inv_query.single_mut() {
            Ok(inv) => inv,
            Err(_) => continue,
        };
        if hq_inv.get(&ResourceId("ore".to_string())) < cost_ore {
            toast_queue.0.push("Not enough ore".to_string());
            continue;
        }
        hq_inv.remove(&ResourceId("ore".to_string()), cost_ore);

        let tile_size = cfg.tile_size;
        let spawn_pos = if let Ok(tf) = farm_tf_query.get(inspected) {
            tf.translation + Vec3::new(tile_size * 0.8, 0.0, 0.5)
        } else {
            Vec3::new(0.0, 0.0, 2.5)
        };
        commands.spawn((
            Cultivator {
                state: crate::agriculture::components::CultivatorState::Idle,
                carried_resource: None,
                carried_amount: 0,
                carry_capacity: def.carry_capacity,
            },
            crate::economy::components::Unit,
            Health {
                current: def.hp,
                max: def.hp,
            },
            Transform::from_translation(spawn_pos),
        ));
        toast_queue.0.push("Cultivator recruited".to_string());
    }
}

pub fn update_farm_crop_text(
    panel: Res<BuildingPanel>,
    farm_query: Query<&Farm>,
    crop_registry: Res<CropRegistry>,
    mut crop_text: Query<&mut Text, With<FarmCropText>>,
) {
    let Some(inspected) = panel.inspected else {
        return;
    };
    if let Ok(farm) = farm_query.get(inspected) {
        if let Ok(mut ct) = crop_text.single_mut() {
            let names: Vec<&str> = farm
                .crop_types
                .iter()
                .map(|c| crop_registry.get(c).map(|d| d.name.as_str()).unwrap_or(c))
                .collect();
            ct.0 = format!("Crops:  {}", names.join(", "));
        }
    }
}

pub fn update_farm_cultivator_count(
    cultivator_query: Query<&Cultivator>,
    mut count_text: Query<&mut Text, With<FarmCultivatorCountText>>,
) {
    if let Ok(mut ct) = count_text.single_mut() {
        ct.0 = format!("Cultivators:  {}", cultivator_query.iter().count());
    }
}

// ── Cleanup on state exit ──

pub fn cleanup_popup(mut commands: Commands, query: Query<Entity, With<PanelModal>>) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
}
