use bevy::prelude::*;
use crate::ui::context::UiDataContext;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::theme::Theme;
use crate::ui::registry::ComponentRegistry;

pub struct OverlayComponent;
impl UiComponent for OverlayComponent {
    fn id(&self) -> &str { "overlay" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &UiDataContext, _theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        let effect = config.get("effect").and_then(|v| v.as_str()).unwrap_or("none");
        let opacity = config.get("opacity").and_then(|v| v.as_float()).unwrap_or(0.25) as f32;

        let bg = match effect {
            "scanlines" => Color::srgba(0.0, 0.0, 0.0, opacity * 0.3),
            "vignette" => Color::srgba(0.0, 0.0, 0.0, opacity * 0.5),
            _ => Color::srgba(0.0, 0.0, 0.0, 0.0),
        };

        let container = spawn_child(commands, parent, (
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO, right: Val::ZERO,
                top: Val::ZERO, bottom: Val::ZERO,
                ..default()
            },
            BackgroundColor(bg),
            Pickable::IGNORE,
        ));

        if effect == "scanlines" {
            commands.entity(container).with_children(|p| {
                for i in 0..120 {
                    p.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::ZERO,
                            right: Val::ZERO,
                            top: Val::Px(i as f32 * 6.0),
                            height: Val::Px(1.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, opacity * 0.6)),
                    ));
                }
            });
        }

        container
    }
}
