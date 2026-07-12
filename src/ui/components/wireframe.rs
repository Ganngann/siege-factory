use bevy::prelude::*;
use crate::ui::context::UiDataContext;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::theme::Theme;
use crate::ui::registry::ComponentRegistry;
use crate::economy::capsule_status::CapsuleStatusRegistry;
use crate::economy::game_components::CurrentTier;
use crate::economy::components::BuildingPanel;

fn parse_hex(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
    } else {
        None
    }
}

#[derive(Component, Clone)]
pub struct WireframeTierTracker;

pub struct WireframeComponent;
impl UiComponent for WireframeComponent {
    fn id(&self) -> &str { "wireframe" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &UiDataContext, _theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        let width = config.get("width").and_then(|v| v.as_float()).unwrap_or(400.0) as f32;
        let height = config.get("height").and_then(|v| v.as_float()).unwrap_or(200.0) as f32;
        let shapes = config.get("shapes").and_then(|v| v.as_array());

        let container = spawn_child(commands, parent, (
            Node {
                width: Val::Px(width),
                height: Val::Px(height),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.06, 1.0)),
            WireframeTierTracker,
        ));

        if let Some(arr) = shapes {
            for s in arr {
                let stype = s.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let ckey = s.get("color_key").and_then(|v| v.as_str()).unwrap_or("#ffffff");
                let color = parse_hex(ckey).unwrap_or(Color::srgb(0.60, 0.60, 0.75));
                match stype {
                    "rect" => {
                        let x = s.get("x").and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                        let y = s.get("y").and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                        let w = s.get("w").and_then(|v| v.as_float()).unwrap_or(10.0) as f32;
                        let h = s.get("h").and_then(|v| v.as_float()).unwrap_or(10.0) as f32;
                        commands.entity(container).with_children(|p| {
                            p.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(x),
                                    top: Val::Px(y),
                                    width: Val::Px(w),
                                    height: Val::Px(h),
                                    ..default()
                                },
                                BackgroundColor(color),
                            ));
                        });
                    },
                    "ellipse" => {
                        let cx = s.get("cx").and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                        let cy = s.get("cy").and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                        let rx = s.get("rx").and_then(|v| v.as_float()).unwrap_or(10.0) as f32;
                        let ry = s.get("ry").and_then(|v| v.as_float()).unwrap_or(10.0) as f32;
                        commands.entity(container).with_children(|p| {
                            p.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(cx - rx),
                                    top: Val::Px(cy - ry),
                                    width: Val::Px(rx * 2.0),
                                    height: Val::Px(ry * 2.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                                BorderColor::all(color),
                            ));
                        });
                    },
                    "vline" => {
                        let x = s.get("x").and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                        let y1 = s.get("y1").and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                        let y2 = s.get("y2").and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                        commands.entity(container).with_children(|p| {
                            p.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(x),
                                    top: Val::Px(y1.min(y2)),
                                    width: Val::Px(1.0),
                                    height: Val::Px((y2 - y1).abs()),
                                    ..default()
                                },
                                BackgroundColor(color),
                            ));
                        });
                    },
                    "hline" => {
                        let y = s.get("y").and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                        let x1 = s.get("x1").and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                        let x2 = s.get("x2").and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                        commands.entity(container).with_children(|p| {
                            p.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(x1.min(x2)),
                                    top: Val::Px(y),
                                    width: Val::Px((x2 - x1).abs()),
                                    height: Val::Px(1.0),
                                    ..default()
                                },
                                BackgroundColor(color),
                            ));
                        });
                    },
                    _ => {},
                }
            }
        }

        container
    }
}

pub fn update_capsule_wireframe_system(
    status_registry: Res<CapsuleStatusRegistry>,
    tier_q: Query<&CurrentTier, With<crate::economy::game_components::Capsule>>,
    panel: Res<BuildingPanel>,
    mut q: Query<&mut BackgroundColor, With<WireframeTierTracker>>,
) {
    let Some(inspected) = panel.inspected else { return; };
    let Ok(tier) = tier_q.get(inspected) else { return; };
    let color_hex = status_registry.status_color("power", tier.0);
    let color = parse_hex(&color_hex).unwrap_or(Color::srgb(1.0, 0.3, 0.2));
    for mut bg in q.iter_mut() {
        bg.0 = color;
    }
}
