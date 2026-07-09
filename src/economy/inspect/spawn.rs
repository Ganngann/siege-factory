use crate::core::utils::silent_despawn;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{
    AlertText, Building, BuildingPanel, CapacityBarFill, CapacityBarText, ConnectionRowText,
    FarmCropSelectButton, FarmCropText, FarmCultivatorCountText, FarmRecruitButton, FlowInputText,
    FlowOutputText, FuelBarBg, FuelBarFill, HpBarFill, HpText, PanelOverlay, PowerStatusText,
    ProgressBarBg, ProgressBarFill, RecipeChangeButton, RecipeNameText, SorterInvertButton,
    SorterResourceButton, StatRowText, StatusText,
};
use crate::economy::resource::ResourceRegistry;
use crate::economy::tiered_structure::ProgressionLogRegistry;
use crate::economy::ui_components::{UpgradeButton, UpgradeInfoText};
use crate::economy::window::{
    ACCENT, BAR_BG, BG_SECTION, HP_GREEN, TEXT_PRIMARY, TEXT_SECONDARY, spawn_window,
};
use bevy::prelude::*;

// 📏 IA NOTE: ce fichier fait 700+ lignes. Si tu le modifies, vérifie que
// tu ne dupliques pas une fonction déjà présente ailleurs dans ce fichier.
// Envisage de le scinder (ex: spawn_panel_ui dans un fichier séparé).
const BUILDING_KIND_FARM: &str = "farm";
const BUILDING_KIND_SORTER: &str = "sorter";

// ⚠️ IA ATTENTION: cette liste de strings doit rester synchronisée avec buildings.toml.
// Si tu ajoutes un building avec des recettes, ajoute son kind ici.
// Solution future: ajouter `has_recipes = true` dans BuildingDef du TOML
// et remplacer cette fonction par `registry.get(kind).map(|d| d.has_recipes).unwrap_or(false)`.
fn kind_has_recipes(kind: &str) -> bool {
    matches!(
        kind,
        "assembler"
            | "assembler_ii"
            | "assembler_iii"
            | "furnace"
            | "furnace_ii"
            | "blast_furnace"
            | "assembly_crane"
            | "alchemy_lab"
            | "electronics_lab"
            | "foundry"
            | "guild_hall"
            | "enchanting_array"
            | "pumpjack"
    )
}

// SUGGEST: extraire dans un struct (clippy::too_many_arguments)
pub fn open_panel(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    entity: Entity,
    building: &Building,
    kind: &str,
    resource_registry: &ResourceRegistry,
    reg: &BuildingRegistry,
    farm_crop_types: Vec<String>,
) {
    // Close existing panel first
    if let Some(e) = panel.root.take() {
        silent_despawn(&mut commands, e);
    }
    if let Some(e) = panel.overlay.take() {
        silent_despawn(&mut commands, e);
    }
    if let Some(e) = panel.recipe_selector.take() {
        silent_despawn(&mut commands, e);
    }
    panel.inspected = None;
    panel.dirty = false;

    let modal_size = Vec2::new(super::MODAL_WIDTH, super::MODAL_HEIGHT);
    let show_recipes = kind_has_recipes(kind);
    let is_farm = kind == BUILDING_KIND_FARM;

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
        resource_registry,
        reg,
        farm_crop_types,
    );

    commands.entity(overlay).add_child(root);
    panel.overlay = Some(overlay);
    panel.root = Some(root);
    panel.inspected = Some(entity);
    panel.dirty = true;
}

// SUGGEST: extraire dans un struct (clippy::too_many_arguments)
fn spawn_panel_ui(
    commands: &mut Commands,
    modal_size: Vec2,
    entity: Entity,
    building: &Building,
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
        &format!("{}  #{}", building.name, entity.to_bits() % 1000),
        modal_size.x,
        modal_size.y,
        x,
        y,
        None,
        |parent| {
            spawn_status_bar(parent);

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
                            spawn_section(left, "FLOW", spawn_flow_content);
                            spawn_section(left, "INVENTORY", |sec| {
                                spawn_inventory_content(sec, entity);
                            });
                            spawn_section(left, "CONNECTIONS", spawn_connections_content);
                        });

                    // ── Right column (Stats + Settings + HP + Alerts) ──
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

                            // Upgrade section (only if building has an upgrade available)
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

// ── Extracted section helpers ──

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
                TextFont::from_font_size(12.0),
                TextColor(TEXT_SECONDARY),
                Node {
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
            ));
        });
}

fn spawn_flow_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
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
}

fn spawn_inventory_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands, entity: Entity) {
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
        crate::economy::components::InventoryGrid {
            cols: 3,
            rows: 2,
            owner: entity,
        },
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
}

fn spawn_connections_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
    sec.spawn((
        ConnectionRowText,
        Text::new("No connections"),
        TextFont::from_font_size(super::SECTION_FONT_SIZE),
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
            TextFont::from_font_size(super::SECTION_FONT_SIZE),
            TextColor(TEXT_SECONDARY),
            Node {
                margin: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
        ));
    }
}

fn spawn_power_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
    sec.spawn((
        PowerStatusText,
        Text::new("Power: --"),
        TextFont::from_font_size(super::SECTION_FONT_SIZE),
        TextColor(TEXT_SECONDARY),
    ));
}

fn spawn_recipe_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
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
}

fn spawn_farm_content(
    sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    farm_crop_types: Vec<String>,
) {
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
        .with_children(|btn| {
            btn.spawn((
                Text::new(crop_type),
                TextFont::from_font_size(super::SECTION_FONT_SIZE),
                TextColor(TEXT_SECONDARY),
            ));
        });
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
}

fn spawn_sorter_content(
    sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    resource_registry: &ResourceRegistry,
) {
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
        .keys().cloned()
        .collect();
    resources.sort();
    for res in &resources {
        sec.spawn((
            SorterResourceButton {
                resource: crate::economy::resource::ResourceId(res.clone()),
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
        .with_children(|btn| {
            btn.spawn((
                Text::new(res),
                TextFont::from_font_size(super::SECTION_FONT_SIZE),
                TextColor(TEXT_SECONDARY),
            ));
        });
    }
}

fn spawn_hp_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
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
}

fn spawn_alerts_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
    sec.spawn((
        AlertText,
        Text::new("No alerts"),
        TextFont::from_font_size(super::SECTION_FONT_SIZE),
        TextColor(TEXT_SECONDARY),
    ));
}

fn spawn_upgrade_section(
    sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    target_kind: &str,
    target_def: &crate::economy::building::BuildingDef,
) {
    let cost_str: String = target_def
        .cost
        .iter()
        .map(|c| format!("{} x{}", c.resource.0, c.amount))
        .collect::<Vec<_>>()
        .join(", ");

    sec.spawn((
        UpgradeInfoText,
        Text::new(format!("Upgrade to: {}", target_def.name)),
        TextFont::from_font_size(12.0),
        TextColor(TEXT_PRIMARY),
        Node {
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        },
    ));
    sec.spawn((
        Text::new(format!("Cost: {}", cost_str)),
        TextFont::from_font_size(super::SECTION_FONT_SIZE),
        TextColor(TEXT_SECONDARY),
        Node {
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        },
    ));
    sec.spawn((
        UpgradeButton {
            target_kind: target_kind.to_string(),
        },
        Button,
        Node {
            width: Val::Px(160.0),
            height: Val::Px(super::CLOSE_BUTTON_SIZE),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.25, 0.45, 0.65)),
    ))
    .with_children(|btn| {
        btn.spawn((
            Text::new("[Upgrade]"),
            TextFont::from_font_size(12.0),
            TextColor(TEXT_PRIMARY),
        ));
    });
}

fn spawn_burner_content(sec: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
    sec.spawn((
        Text::new("Combustion"),
        TextFont::from_font_size(10.0),
        TextColor(TEXT_SECONDARY),
    ));
    sec.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(14.0),
            ..default()
        },
        BackgroundColor(BAR_BG),
        FuelBarBg,
    ))
    .with_children(|bg| {
        bg.spawn((
            Node {
                width: Val::Percent(0.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(ACCENT),
            FuelBarFill,
        ));
    });
}

// ── Section wrapper ──

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

// ── Capsule progression panel ──

pub fn open_capsule_panel(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    entity: Entity,
    building: &Building,
    reg: &BuildingRegistry,
    logs: &ProgressionLogRegistry,
    tier_index: usize,
) {
    // Close existing panel
    if let Some(e) = panel.root.take() {
        silent_despawn(&mut commands, e);
    }
    if let Some(e) = panel.overlay.take() {
        silent_despawn(&mut commands, e);
    }
    panel.inspected = None;

    let Some(def) = reg.get(&building.kind) else { return };
    let total_tiers = def.tiers.len();

    // Build content lines
    let mut tier_lines: Vec<String> = Vec::new();
    tier_lines.push(format!("CAPSULE GENESIS — Progression"));
    tier_lines.push(String::new());

    for i in 0..total_tiers {
        let tier_def = &def.tiers[i];
        let log_title = logs
            .logs
            .iter()
            .find(|l| l.id.as_str() == tier_def.log_id.as_deref().unwrap_or(""))
            .map(|l| l.title.as_str())
            .unwrap_or(&tier_def.texture);
        let prefix = if i < tier_index {
            "✅"
        } else if i == tier_index {
            "◉"
        } else {
            "○"
        };
        let status = if i < tier_index {
            " (complété)"
        } else if i == tier_index {
            " (en cours)"
        } else {
            ""
        };
        tier_lines.push(format!(" {} Tier {} — {}{}", prefix, i, log_title, status));
    }

    tier_lines.push(String::new());

    // Current tier requirements
    if tier_index < total_tiers {
        let current = &def.tiers[tier_index];
        if !current.required_items.is_empty() {
            tier_lines.push("Items requis :".to_string());
            for (res, amt) in &current.required_items {
                tier_lines.push(format!("  {} {}/{}", res.display_name(), 0, amt));
            }
            tier_lines.push(String::new());
            tier_lines.push("(Appuyez sur E à côté de la capsule)".to_string());
        }

        // Log text for current tier
        if let Some(ref log_id) = current.log_id {
            if let Some(entry) = logs.logs.iter().find(|l| l.id == *log_id) {
                tier_lines.push(String::new());
                tier_lines.push(format!("\"{}\"", entry.text));
            }
        }
    }

    let full_text = tier_lines.join("\n");

    // Build the UI
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

    let root = spawn_window(
        &mut commands,
        &format!("Capsule — {}", building.name),
        super::DEPOSIT_MODAL_WIDTH,
        super::DEPOSIT_MODAL_HEIGHT,
        120.0,
        80.0,
        None,
        |parent| {
            parent.spawn((
                Text::new(full_text),
                TextFont::from_font_size(12.0),
                TextColor(TEXT_PRIMARY),
                Node {
                    padding: UiRect::all(Val::Px(12.0)),
                    ..default()
                },
            ));
        },
    );

    panel.overlay = Some(overlay);
    panel.root = Some(root);
    panel.inspected = Some(entity);
    panel.dirty = false;
}
