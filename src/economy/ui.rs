use bevy::prelude::*;
use crate::economy::building::BuildingRegistry;
use crate::economy::resource::{ResourceId, Inventory};
use crate::economy::systems::{BeltDirection, BuildMode, BuildKind, HQ};
use crate::economy::unit_config::UnitConfig;
use crate::rendering::direction_arrow;

#[derive(Component)]
pub struct BuildModeText;

fn building_key(kind: BuildKind) -> &'static str {
    match kind {
        BuildKind::Miner => "1",
        BuildKind::Assembler => "2",
        BuildKind::Belt => "3",
        BuildKind::Wall => "4",
        BuildKind::Turret => "5",
    }
}

fn cost_string(cost: &[crate::economy::building::BuildingCost]) -> String {
    if cost.is_empty() {
        "free".to_string()
    } else {
        cost.iter()
            .map(|c| format!("{} {:?}", c.amount, c.resource))
            .collect::<Vec<_>>()
            .join(" + ")
    }
}

fn kind_name(kind: BuildKind) -> &'static str {
    match kind {
        BuildKind::Miner => "Miner (on brown deposit)",
        BuildKind::Assembler => "Assembler",
        BuildKind::Belt => "Belt",
        BuildKind::Wall => "Wall",
        BuildKind::Turret => "Turret",
    }
}

fn unit_cost_str(cost: &[crate::economy::unit_config::UnitCost]) -> String {
    cost.iter()
        .map(|c| format!("{} {:?}", c.amount, c.resource))
        .collect::<Vec<_>>()
        .join(" + ")
}

fn building_id_to_key(id: &str) -> &'static str {
    match id {
        "miner" => "1",
        "assembler" => "2",
        "belt" => "3",
        "wall" => "4",
        "turret" => "5",
        _ => "?",
    }
}

pub fn build_mode_indicator(
    build_mode: Res<BuildMode>,
    belt_dir: Res<BeltDirection>,
    registry: Res<BuildingRegistry>,
    unit_cfg: Res<UnitConfig>,
    mut text_query: Query<(Entity, &mut Text), With<BuildModeText>>,
    mut commands: Commands,
) {
    let msg = match build_mode.0 {
        Some(kind) => {
            let key = building_key(kind);
            let dir = if kind == BuildKind::Belt {
                format!(" [{}]  R: rotate", direction_arrow(belt_dir.0))
            } else {
                String::new()
            };
            format!("BUILD: [{}] {}{}  (click tile  |  Esc to cancel)", key, kind_name(kind), dir)
        }
        None => {
            let mut lines = vec!["--- BUILD ---".to_string()];
            for b in &registry.buildings {
                let key = building_id_to_key(&b.id);
                lines.push(format!("[{}] {}  ({})", key, b.name, cost_string(&b.cost)));
            }
            lines.push("--- UNITS ---".to_string());
            let s_cost = unit_cost_str(&unit_cfg.soldier.unit.cost);
            let w_cost = unit_cost_str(&unit_cfg.worker.unit.cost);
            lines.push(format!("[6] Soldier  ({})", s_cost));
            lines.push(format!("[7] Worker  ({})", w_cost));
            lines.push("Esc: Pause".to_string());
            lines.join("\n")
        }
    };

    if let Ok((_, mut text)) = text_query.get_single_mut() {
        text.sections[0].value = msg;
    } else {
        commands.spawn((
            BuildModeText,
            TextBundle {
                text: Text::from_sections([TextSection::new(
                    msg,
                    TextStyle { font_size: 14.0, color: Color::srgb(1.0, 0.9, 0.4), ..default() },
                )]),
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
        ));
    }
}

pub fn ore_count_ui(
    hq_query: Query<&Inventory, With<HQ>>,
    mut text_query: Query<(Entity, &mut Text), With<OreCountText>>,
    mut commands: Commands,
) {
    let ore = hq_query
        .get_single()
        .map(|inv| inv.get(ResourceId::Ore))
        .unwrap_or(0);
    let ammo = hq_query
        .get_single()
        .map(|inv| inv.get(ResourceId::Ammo))
        .unwrap_or(0);
    let energy = hq_query
        .get_single()
        .map(|inv| inv.get(ResourceId::Energy))
        .unwrap_or(0);

    let msg = format!("Ore: {ore}  Ammo: {ammo}  Energy: {energy}");

    if let Ok((_, mut text)) = text_query.get_single_mut() {
        text.sections[0].value = msg;
    } else {
        commands.spawn((
            OreCountText,
            TextBundle {
                text: Text::from_sections([TextSection::new(
                    msg,
                    TextStyle { font_size: 18.0, color: Color::WHITE, ..default() },
                )]),
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
        ));
    }
}

#[derive(Component)]
pub struct OreCountText;
