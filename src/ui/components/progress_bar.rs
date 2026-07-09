use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct ProgressBarComponent;
impl UiComponent for ProgressBarComponent {
    fn id(&self) -> &str { "progress_bar" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, _registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let key = config.get("key").and_then(|v| v.as_str()).unwrap_or("0");
        let max_key = config.get("max_key").and_then(|v| v.as_str()).unwrap_or("100");
        let current: f32 = data.resolve(key).parse().unwrap_or(0.0);
        let max: f32 = data.resolve(max_key).parse().unwrap_or(100.0);
        let pct = if max > 0.0 { (current / max * 100.0).min(100.0) } else { 0.0 };
        let container = spawn_child(commands, parent, (Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0), ..default() }, BackgroundColor(Color::NONE)));
        commands.entity(container).with_children(|p| {
            p.spawn((Node { width: Val::Percent(100.0), height: Val::Px(14.0), ..default() }, BackgroundColor(theme.bar_bg))).with_children(|bg| { bg.spawn((Node { width: Val::Percent(pct), height: Val::Percent(100.0), ..default() }, BackgroundColor(theme.progress_fill))); });
            p.spawn((Text::new(format!("{:.0}/{:.0}", current, max)), tf(theme.font_size_small), TextColor(theme.text_secondary)));
        });
        container
    }
}
