use bevy::prelude::*;
use crate::core::tooltip::TooltipText;
use crate::economy::building::BuildingRegistry;
use crate::economy::resource::Inventory;
use crate::economy::components::{BuildMode, HQ};
use crate::economy::unit_config::UnitConfig;
use crate::unit::SpawnUnitEvent;

#[derive(Component)]
pub struct BuildBarPanel;

pub enum BuildBarEntryKind {
    Building(String),
    Unit(String),
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

pub fn spawn_build_bar(
    mut commands: Commands,
    registry: Res<BuildingRegistry>,
    unit_cfg: Res<UnitConfig>,
) {
    commands
        .spawn((
            BuildBarPanel,
            Node {
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
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.85)),
        ))
        .with_children(|parent| {
            for def in &registry.buildings {
                if def.id == "hq" {
                    continue;
                }
                let kind = def.id.clone();
                let bg_color = def.color;
                let cost_str = format_building_cost(&def.cost);
                parent
                    .spawn((
                        BuildBarEntry { kind: BuildBarEntryKind::Building(kind), original_color: bg_color },
                        Button,
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(64.0),
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
                        b.spawn((
                            Text::new(&def.name),
                            TextFont::from_font_size(13.0),
                            TextColor(Color::WHITE),
                        ));
                        b.spawn((
                            Text::new(cost_str),
                            TextFont::from_font_size(10.0),
                            TextColor(Color::srgb(1.0, 0.85, 0.3)),
                        ));
                    });
            }

            parent.spawn((
                Text::new("|"),
                TextFont::from_font_size(20.0),
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.3)),
            ));

            for (id, def) in &unit_cfg.units {
                let bg_color = Color::srgb(0.2, 0.25, 0.3);
                let cost_str = format_unit_cost(&def.cost);
                parent
                    .spawn((
                        BuildBarEntry { kind: BuildBarEntryKind::Unit(id.clone()), original_color: bg_color },
                        Button,
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(64.0),
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
                        b.spawn((
                            Text::new(&def.name),
                            TextFont::from_font_size(13.0),
                            TextColor(Color::WHITE),
                        ));
                        b.spawn((
                            Text::new(cost_str),
                            TextFont::from_font_size(10.0),
                            TextColor(Color::srgb(1.0, 0.85, 0.3)),
                        ));
                    });
            }
        });
}

pub fn build_bar_interaction(
    query: Query<(&Interaction, &BuildBarEntry), Changed<Interaction>>,
    mut build_mode: ResMut<BuildMode>,
    mut commands: Commands,
    mut tooltip: ResMut<TooltipText>,
    registry: Res<BuildingRegistry>,
    unit_cfg: Res<UnitConfig>,
) {
    for (interaction, entry) in &query {
        match *interaction {
            Interaction::Pressed => {
                match &entry.kind {
                    BuildBarEntryKind::Building(kind) => {
                        build_mode.0 = match &build_mode.0 {
                            Some(current) if current == kind => None,
                            _ => Some(kind.clone()),
                        };
                    }
                    BuildBarEntryKind::Unit(kind) => {
                        commands.trigger(SpawnUnitEvent(kind.clone()));
                    }
                }
            }
            Interaction::Hovered => {
                match &entry.kind {
                    BuildBarEntryKind::Building(kind) => {
                        let key = registry.buildings.iter()
                            .filter(|b| b.id != "hq")
                            .position(|b| &b.id == kind)
                            .map(|i| format!("[{}]", i + 1))
                            .unwrap_or_default();
                        let def = registry.buildings.iter()
                            .find(|def| &def.id == kind);
                        if let Some(d) = def {
                            let cost = format_building_cost(&d.cost);
                            let mut parts = vec![
                                format!("{} {}  HP:{}  Cost:{}", key, d.name, d.hp, cost)
                            ];
                            if d.requires_deposit {
                                parts.push("Requires ore deposit".to_string());
                            }
                            if let Some(ref p) = d.production {
                                parts.push(format!("Produces {:?} every {:.1}s", p.resource, p.interval_sec));
                            }
                            if let Some(ref b) = d.belt {
                                parts.push(format!("{} slots, speed {:.1}", b.slots, b.speed));
                            }
                            if let Some(ref c) = d.combat {
                                parts.push(format!("Dmg {}  Range {:.0}  Rate {:.1}s", c.damage, c.range.sqrt(), c.fire_rate_sec));
                            }
                            tooltip.0 = Some(parts.join("  |  "));
                        }
                    }
                    BuildBarEntryKind::Unit(kind) => {
                        let keys: Vec<&String> = unit_cfg.units.keys().collect();
                        let key = keys.iter()
                            .position(|k| *k == kind)
                            .map(|i| format!("[{}]", i + 6))
                            .unwrap_or_default();
                        let def = unit_cfg.get(kind);
                        if let Some(d) = def {
                            let cost = format_unit_cost(&d.cost);
                            let mut parts = vec![
                                format!("{} {}  HP:{}  Cost:{}", key, d.name, d.hp, cost)
                            ];
                            if d.kind == "combat" {
                                parts.push(format!("Dmg {}  Range {:.0}  Rate {:.1}s", d.damage, d.range_tiles, d.fire_rate_sec));
                            } else if d.kind == "harvester" {
                                parts.push(format!("Speed {:.0}  Mine interval {:.1}s", d.speed, d.mine_interval_sec));
                            }
                            tooltip.0 = Some(parts.join("  |  "));
                        }
                    }
                }
            }
            Interaction::None => {
                tooltip.0 = None;
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
    let hq_inv = hq_query.single().ok();

    for (entry, mut bg, mut border) in button_query.iter_mut() {
        match &entry.kind {
            BuildBarEntryKind::Building(kind) => {
                let is_active = build_mode.0.as_ref() == Some(kind);
                let affordable = hq_inv
                    .and_then(|inv| {
                        registry.buildings.iter()
                            .find(|def| &def.id == kind)
                            .map(|def| can_afford_building(inv, &def.cost))
                    })
                    .unwrap_or(false);

                *border = BorderColor::all(if is_active {
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
                let affordable = hq_inv
                    .and_then(|inv| unit_cfg.get(kind).map(|def| can_afford_unit(inv, &def.cost)))
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
