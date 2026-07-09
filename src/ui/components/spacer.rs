use bevy::prelude::*;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct SpacerComponent;
impl UiComponent for SpacerComponent {
    fn id(&self) -> &str { "spacer" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, _registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let h = config.get("height").and_then(|v| v.as_float()).unwrap_or(8.0) as f32;
        spawn_child(commands, parent, (Node { height: Val::Px(h), ..default() }, BackgroundColor(Color::NONE)))
    }
}
