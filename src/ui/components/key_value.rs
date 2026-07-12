use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::context::UiDataContext;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::theme::Theme;
use crate::ui::registry::ComponentRegistry;

fn parse_hex_color(hex: &str) -> Option<Color> {
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

pub struct KeyValueComponent;
impl UiComponent for KeyValueComponent {
    fn id(&self) -> &str { "key_value" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, data: &UiDataContext, theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        let key = config.get("key").and_then(|v| v.as_str()).unwrap_or("");
        let value_key = config.get("value_key").and_then(|v| v.as_str()).unwrap_or("");
        let value = data.resolve(value_key);

        let row = spawn_child(commands, parent, (
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                padding: UiRect::vertical(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ));
        commands.entity(row).with_children(|p| {
            p.spawn((Text::new(format!("{}:", key)), tf(theme.font_size_small), TextColor(theme.text_secondary)));
            p.spawn((Text::new(value), tf(theme.font_size_small), TextColor(Color::srgb(0.60, 0.60, 0.75))));
        });
        row
    }
}

pub struct KeyValueListComponent;
impl UiComponent for KeyValueListComponent {
    fn id(&self) -> &str { "key_value_list" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &UiDataContext, _theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        let items = config.get("items").and_then(|v| v.as_array());

        let container = spawn_child(commands, parent, (
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            CapsuleStatusList,
        ));

        if let Some(arr) = items {
            for item in arr {
                let key = item.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let value_key = item.get("value_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let system_id = value_key.trim_start_matches("capsule.status_").to_string();
                commands.entity(container).with_children(|p| {
                    p.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            width: Val::Percent(100.0),
                            padding: UiRect::vertical(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    )).with_children(|row| {
                        row.spawn((Text::new(key), tf(_theme.font_size_small), TextColor(_theme.text_secondary)));
                        row.spawn((
                            Text::new(String::new()),
                            tf(_theme.font_size_small),
                            TextColor(Color::srgb(0.60, 0.60, 0.75)),
                            CapsuleStatusRow { system_id },
                        ));
                    });
                });
            }
        }

        container
    }
}

#[derive(Component, Clone)]
pub struct CapsuleStatusList;

#[derive(Component, Clone)]
pub struct CapsuleStatusRow {
    pub system_id: String,
}

pub fn update_capsule_statuses_system(
    status_registry: Res<crate::economy::capsule_status::CapsuleStatusRegistry>,
    tier_q: Query<&crate::economy::game_components::CurrentTier, With<crate::economy::game_components::Capsule>>,
    panel: Res<crate::economy::components::BuildingPanel>,
    mut q: Query<(&CapsuleStatusRow, &mut Text, &mut TextColor)>,
) {
    let Some(inspected) = panel.inspected else { return; };
    let Ok(tier) = tier_q.get(inspected) else { return; };
    for (row, mut text, mut color) in q.iter_mut() {
        text.0 = status_registry.status_text(&row.system_id, tier.0);
        if let Some(c) = parse_hex_color(&status_registry.status_color(&row.system_id, tier.0)) {
            color.0 = c;
        }
    }
}
