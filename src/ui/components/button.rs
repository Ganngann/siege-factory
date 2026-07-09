use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct ButtonComponent;
impl UiComponent for ButtonComponent {
    fn id(&self) -> &str { "button" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, _registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let text = config.get("text").and_then(|v| v.as_str()).unwrap_or("Button").to_string();
        let child = spawn_child(commands, parent, (Button, Node { width: Val::Px(120.0), height: Val::Px(24.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, margin: UiRect::vertical(Val::Px(2.0)), ..default() }, BackgroundColor(theme.btn_inactive)));
        commands.entity(child).with_children(|p| { p.spawn((Text::new(text), tf(theme.font_size_body), TextColor(theme.text_primary))); });
        child
    }
}
