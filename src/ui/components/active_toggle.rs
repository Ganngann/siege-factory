use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct ActiveToggleComponent;
impl UiComponent for ActiveToggleComponent {
    fn id(&self) -> &str { "active_toggle" }
    fn render(&self, commands: &mut Commands, parent: Entity, _config: &toml::Value, data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, _registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let is_on = data.resolve("active") == "ON";
        let label = if is_on { "⏸ Pause" } else { "▶ Activer" };
        let bg = if is_on { theme.btn_active } else { theme.btn_inactive };
        let child = spawn_child(commands, parent, (crate::economy::components::ActiveToggleButton, Button, Node { width: Val::Percent(100.0), height: Val::Px(26.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, margin: UiRect::vertical(Val::Px(2.0)), ..default() }, BackgroundColor(bg)));
        commands.entity(child).with_children(|btn| { btn.spawn((Text::new(label.to_string()), tf(theme.font_size_body), TextColor(theme.text_primary))); });
        child
    }
}
