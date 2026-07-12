use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::context::UiDataContext;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::theme::Theme;
use crate::ui::registry::ComponentRegistry;

pub struct AlertHeaderComponent;
impl UiComponent for AlertHeaderComponent {
    fn id(&self) -> &str { "alert_header" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, data: &UiDataContext, theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        let title = config.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let alert = config.get("alert").and_then(|v| v.as_str()).unwrap_or("");
        let subtitle_key = config.get("subtitle_key").and_then(|v| v.as_str()).unwrap_or("");
        let subtitle = data.resolve(subtitle_key);

        let container = spawn_child(commands, parent, (
            Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0), ..default() },
            BackgroundColor(Color::NONE),
        ));

        commands.entity(container).with_children(|p| {
            p.spawn((Text::new(title.to_string()), tf(theme.font_size_medium), TextColor(theme.text_primary)));
            if !alert.is_empty() {
                p.spawn((
                    Node { padding: UiRect::all(Val::Px(4.0)), ..default() },
                    BackgroundColor(Color::srgb(0.60, 0.15, 0.15)),
                )).with_children(|banner| {
                    banner.spawn((Text::new(format!("█ {} █", alert)), tf(theme.font_size_small), TextColor(Color::WHITE)));
                });
            }
            if !subtitle.is_empty() {
                p.spawn((Text::new(subtitle), tf(theme.font_size_small), TextColor(theme.text_secondary)));
            }
        });

        container
    }
}
