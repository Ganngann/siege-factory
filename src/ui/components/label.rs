use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct LabelComponent;
impl UiComponent for LabelComponent {
    fn id(&self) -> &str { "label" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, _registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let text = config.get("text").and_then(|v| v.as_str()).unwrap_or("");
        let style = config.get("style").and_then(|v| v.as_str()).unwrap_or("body");
        let (size, color) = match style {
            "title" => (theme.font_size_title, theme.text_primary),
            "small" => (theme.font_size_small, theme.text_secondary),
            "green" => (theme.font_size_body, theme.text_green),
            _ => (theme.font_size_body, theme.text_primary),
        };
        spawn_child(commands, parent, (Text::new(text.to_string()), tf(size), TextColor(color), Node { margin: UiRect::bottom(Val::Px(2.0)), ..default() }))
    }
}
