use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct DataLabelComponent;
impl UiComponent for DataLabelComponent {
    fn id(&self) -> &str { "data_label" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, _registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let key = config.get("key").and_then(|v| v.as_str()).unwrap_or("");
        let value = data.resolve(key);
        let style = config.get("style").and_then(|v| v.as_str()).unwrap_or("body");
        let color = match style { "green" => theme.text_green, "yellow" => theme.text_yellow, _ => theme.text_primary };
        spawn_child(commands, parent, (Text::new(value), tf(theme.font_size_body), TextColor(color)))
    }
}
