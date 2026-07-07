use crate::agriculture::components::{CropRegistry, Cultivator, Farm};
use crate::economy::belt::BeltSlots;
use crate::economy::components::{
    Active, ActiveToggleButton, AlertText, Assembler, Building, BuildingPanel, BuildingTitleText,
    BurnerGenerator, CapacityBarFill, CapacityBarText, ConnectionRowText, FarmCropText,
    FarmCultivatorCountText, FlowInputText, FlowOutputText, FuelBarFill, HpBarFill, HpText,
    PowerConsumer, PowerStatusText, ProgressBarFill, RecipeNameText, StatRowText, StatusText,
};
use crate::economy::recipe::RecipeRegistry;
use crate::economy::resource::{Inventory, ResourceRegistry};
use crate::economy::window::{BTN_ACTIVE, BTN_INACTIVE};
use crate::enemy::components::Health;
use bevy::prelude::*;

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
                            let name = resource_registry.display_name(rid);
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
                        let name = resource_registry.display_name(rid);
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
                let name = resource_registry.display_name(rid);
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

pub fn update_panel_power(
    panel: Res<BuildingPanel>,
    power_query: Query<Option<&PowerConsumer>>,
    mut power_text: Query<&mut Text, With<PowerStatusText>>,
) {
    let Some(inspected) = panel.inspected else {
        return;
    };
    let Ok(power) = power_query.get(inspected) else {
        return;
    };
    let Ok(mut pt) = power_text.single_mut() else {
        return;
    };

    let msg = match power {
        Some(pc) if pc.satisfied => format!("Power: OK ({:.0} kW)", pc.draw),
        Some(_) => "Power: NO POWER".to_string(),
        None => "Power: --".to_string(),
    };
    pt.0 = msg;
}

pub fn update_panel_burner(
    panel: Res<BuildingPanel>,
    burner_query: Query<&BurnerGenerator>,
    mut fill_q: Query<&mut Node, With<FuelBarFill>>,
) {
    let Some(inspected) = panel.inspected else {
        return;
    };
    let Ok(burner) = burner_query.get(inspected) else {
        return;
    };
    let Ok(mut node) = fill_q.single_mut() else {
        return;
    };
    let pct = (burner.fuel_burn_timer / burner.fuel_burn_interval).clamp(0.0, 1.0) * 100.0;
    node.width = Val::Percent(pct);
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
