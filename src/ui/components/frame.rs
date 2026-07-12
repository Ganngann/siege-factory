use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::context::UiDataContext;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::theme::Theme;
use crate::ui::registry::ComponentRegistry;

fn led_color(name: &str) -> Color {
    match name {
        "red" => Color::srgb(1.0, 0.2, 0.2),
        "green" => Color::srgb(0.2, 1.0, 0.2),
        "yellow" => Color::srgb(1.0, 0.8, 0.2),
        "blue" => Color::srgb(0.2, 0.5, 1.0),
        _ => Color::srgb(0.3, 0.3, 0.3),
    }
}

pub struct FrameComponent;
impl UiComponent for FrameComponent {
    fn id(&self) -> &str { "frame" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &UiDataContext, theme: &Theme, registry: &ComponentRegistry) -> Entity {
        let title = config.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let variant = config.get("variant").and_then(|v| v.as_str()).unwrap_or("flat");
        let led = config.get("led").and_then(|v| v.as_str()).unwrap_or("");
        let padding = config.get("padding").and_then(|v| v.as_float()).unwrap_or(8.0) as f32;
        let children = config.get("children").and_then(|v| v.as_array());

        let (bg, border_color) = match variant {
            "terminal" => (Color::srgb(0.05, 0.05, 0.10), Color::srgb(0.30, 0.60, 0.30)),
            "bezel" => (Color::srgb(0.12, 0.12, 0.18), Color::srgb(0.40, 0.40, 0.55)),
            "glow" => (Color::srgb(0.10, 0.10, 0.18), Color::srgb(0.40, 0.60, 1.00)),
            _ => (Color::srgb(0.08, 0.08, 0.14), Color::srgb(0.30, 0.30, 0.40)),
        };

        let container_e = spawn_child(commands, parent, (
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(padding)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(bg),
            BorderColor::all(border_color),
        ));

        if !led.is_empty() {
            commands.entity(container_e).with_children(|p| {
                p.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(6.0),
                        top: Val::Px(6.0),
                        width: Val::Px(8.0),
                        height: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(led_color(led)),
                ));
            });
        }

        if !title.is_empty() {
            commands.entity(container_e).with_children(|p| {
                p.spawn((Text::new(title), tf(theme.font_size_small), TextColor(theme.text_secondary)));
            });
        }

        if let Some(arr) = children {
            for child_config in arr {
                let cid = child_config.get("type").and_then(|v| v.as_str()).unwrap_or("label");
                if let Some(comp) = registry.get(cid) {
                    comp.render(commands, container_e, child_config, _data, theme, registry);
                }
            }
        }

        container_e
    }
}
