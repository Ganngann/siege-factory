use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct ConditionalTextComponent;
impl UiComponent for ConditionalTextComponent {
    fn id(&self) -> &str { "conditional_text" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, _registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let source_key = config.get("source_key").and_then(|v| v.as_str()).unwrap_or("");
        let source_val = data.resolve(source_key);
        let text = config.get("values").and_then(|v| v.as_array())
            .and_then(|arr| arr.iter()
                .find(|v| v.get("when").and_then(|w| w.as_str()) == Some(&source_val))
                .and_then(|v| v.get("text").and_then(|t| t.as_str())))
            .unwrap_or("");
        spawn_child(commands, parent, (Text::new(text.to_string()), tf(theme.font_size_body), TextColor(theme.text_primary)))
    }
}
