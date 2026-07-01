use bevy::prelude::*;
use crate::economy::building::BuildingRegistry;
use crate::economy::resource::Inventory;
use crate::economy::components::{BuildKind, BuildMode, HQ, SetBuildModeEvent};
use crate::economy::unit_config::UnitConfig;
use crate::unit::{SpawnUnitEvent, UnitKind};

#[derive(Component)]
pub struct BuildBarPanel;

pub enum BuildBarEntryKind {
    Building(BuildKind),
    Unit(UnitKind),
}

#[derive(Component)]
pub struct BuildBarEntry {
    pub kind: BuildBarEntryKind,
    pub original_color: Color,
}

fn format_building_cost(cost: &[crate::economy::building::BuildingCost]) -> String {
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

fn can_afford_building(hq_inv: &Inventory, cost: &[crate::economy::building::BuildingCost]) -> bool {
    cost.iter().all(|c| hq_inv.get(c.resource) >= c.amount)
}

fn can_afford_unit(hq_inv: &Inventory, cost: &[crate::economy::unit_config::UnitCost]) -> bool {
    cost.iter().all(|c| hq_inv.get(c.resource) >= c.amount)
}

fn kind_to_id(kind: BuildKind) -> &'static str {
    match kind {
        BuildKind::Miner => "miner",
        BuildKind::Assembler => "assembler",
        BuildKind::Belt => "belt",
        BuildKind::Wall => "wall",
        BuildKind::Turret => "turret",
    }
}

pub fn spawn_build_bar(
    mut commands: Commands,
    registry: Res<BuildingRegistry>,
    unit_cfg: Res<UnitConfig>,
) {
    commands
        .spawn((
            BuildBarPanel,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(80.0),
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    left: Val::Px(0.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(6.0),
                    padding: UiRect::all(Val::Px(6.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.85)),
                ..default()
            },
        ))
        .with_children(|parent| {
            for def in &registry.buildings {
                if def.id == "hq" {
                    continue;
                }
                let kind = match def.id.as_str() {
                    "miner" => BuildKind::Miner,
                    "assembler" => BuildKind::Assembler,
                    "belt" => BuildKind::Belt,
                    "wall" => BuildKind::Wall,
                    "turret" => BuildKind::Turret,
                    _ => continue,
                };
                let bg_color = def.color;
                let cost_str = format_building_cost(&def.cost);
                parent
                    .spawn((
                        BuildBarEntry {
                            kind: BuildBarEntryKind::Building(kind),
                            original_color: bg_color,
                        },
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(100.0),
                                height: Val::Px(64.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            background_color: BackgroundColor(bg_color),
                            border_color: BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                            ..default()
                        },
                    ))
                    .with_children(|b| {
                        b.spawn(TextBundle::from_section(
                            &def.name,
                            TextStyle { font_size: 13.0, color: Color::WHITE, ..default() },
                        ));
                        b.spawn(TextBundle::from_section(
                            cost_str,
                            TextStyle { font_size: 10.0, color: Color::srgb(1.0, 0.85, 0.3), ..default() },
                        ));
                    });
            }

            parent.spawn(TextBundle::from_section(
                "|",
                TextStyle { font_size: 20.0, color: Color::srgba(1.0, 1.0, 1.0, 0.3), ..default() },
            ));

            for (unit_kind, name, cost) in [
                (UnitKind::Soldier, &unit_cfg.soldier.unit.name, format_unit_cost(&unit_cfg.soldier.unit.cost)),
                (UnitKind::Worker, &unit_cfg.worker.unit.name, format_unit_cost(&unit_cfg.worker.unit.cost)),
            ] {
                let bg_color = Color::srgb(0.2, 0.25, 0.3);
                parent
                    .spawn((
                        BuildBarEntry {
                            kind: BuildBarEntryKind::Unit(unit_kind),
                            original_color: bg_color,
                        },
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(100.0),
                                height: Val::Px(64.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            background_color: BackgroundColor(bg_color),
                            border_color: BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                            ..default()
                        },
                    ))
                    .with_children(|b| {
                        b.spawn(TextBundle::from_section(
                            name.as_str(),
                            TextStyle { font_size: 13.0, color: Color::WHITE, ..default() },
                        ));
                        b.spawn(TextBundle::from_section(
                            cost,
                            TextStyle { font_size: 10.0, color: Color::srgb(1.0, 0.85, 0.3), ..default() },
                        ));
                    });
            }
        });
}

pub fn build_bar_interaction(
    query: Query<(&Interaction, &BuildBarEntry), Changed<Interaction>>,
    mut build_events: EventWriter<SetBuildModeEvent>,
    mut unit_events: EventWriter<SpawnUnitEvent>,
) {
    for (interaction, entry) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match &entry.kind {
            BuildBarEntryKind::Building(kind) => {
                build_events.send(SetBuildModeEvent(Some(*kind)));
            }
            BuildBarEntryKind::Unit(kind) => {
                unit_events.send(SpawnUnitEvent(*kind));
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_build_bar(
    build_mode: Res<BuildMode>,
    hq_query: Query<&Inventory, With<HQ>>,
    registry: Res<BuildingRegistry>,
    unit_cfg: Res<UnitConfig>,
    mut button_query: Query<(&BuildBarEntry, &mut BackgroundColor, &mut BorderColor)>,
) {
    let hq_inv = hq_query.get_single().ok();

    for (entry, mut bg, mut border) in button_query.iter_mut() {
        match &entry.kind {
            BuildBarEntryKind::Building(kind) => {
                let is_active = build_mode.0 == Some(*kind);
                let affordable = hq_inv
                    .and_then(|inv| {
                        registry
                            .buildings
                            .iter()
                            .find(|def| def.id == kind_to_id(*kind))
                            .map(|def| can_afford_building(inv, &def.cost))
                    })
                    .unwrap_or(false);

                *border = BorderColor(if is_active {
                    Color::srgb(0.3, 1.0, 0.3)
                } else {
                    Color::srgba(1.0, 1.0, 1.0, 0.2)
                });

                bg.0 = if affordable {
                    entry.original_color
                } else {
                    Color::srgb(0.3, 0.3, 0.3)
                };
            }
            BuildBarEntryKind::Unit(kind) => {
                let cost = match kind {
                    UnitKind::Soldier => &unit_cfg.soldier.unit.cost,
                    UnitKind::Worker => &unit_cfg.worker.unit.cost,
                };
                let affordable = hq_inv
                    .map(|inv| can_afford_unit(inv, cost))
                    .unwrap_or(false);

                bg.0 = if affordable {
                    entry.original_color
                } else {
                    Color::srgb(0.3, 0.3, 0.3)
                };
            }
        }
    }
}

pub fn cleanup_build_bar(mut commands: Commands, query: Query<Entity, With<BuildBarPanel>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
