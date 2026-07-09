use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct HpBarComponent;
impl UiComponent for HpBarComponent {
    fn id(&self) -> &str { "hp_bar" }
    fn render(&self, commands: &mut Commands, parent: Entity, _config: &toml::Value, data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, _registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let current: f32 = data.resolve("hp.current").parse().unwrap_or(0.0);
        let max: f32 = data.resolve("hp.max").parse().unwrap_or(100.0);
        let pct = if max > 0.0 { (current / max * 100.0).min(100.0) } else { 0.0 };
        spawn_child(commands, parent, (Text::new(format!("HP: {:.0}/{:.0}", current, max)), tf(theme.font_size_small), TextColor(theme.text_secondary)))
    }
}
