use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct TierProgressComponent;
impl UiComponent for TierProgressComponent {
    fn id(&self) -> &str { "tier_progress" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, _registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let current_key = config.get("current_key").and_then(|v| v.as_str()).unwrap_or("0");
        let max_key = config.get("max_key").and_then(|v| v.as_str()).unwrap_or("1");
        let cur: f32 = data.resolve(current_key).parse().unwrap_or(0.0);
        let max: f32 = data.resolve(max_key).parse().unwrap_or(1.0);
        let pct = if max > 0.0 { (cur / max * 100.0).min(100.0) } else { 0.0 };
        let bars: String = (0..(max as usize)).map(|i| if i < cur as usize { '█' } else { '░' }).collect();
        let container = spawn_child(commands, parent, (Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0), ..default() }, BackgroundColor(Color::NONE)));
        commands.entity(container).with_children(|p| {
            p.spawn((Node { width: Val::Percent(100.0), height: Val::Px(14.0), ..default() }, BackgroundColor(theme.bar_bg))).with_children(|bg| {
                bg.spawn((Node { width: Val::Percent(pct), height: Val::Percent(100.0), ..default() }, BackgroundColor(theme.progress_fill)));
            });
            p.spawn((Text::new(format!("{}  {:.0}/{:.0}", bars, cur, max)), tf(theme.font_size_small), TextColor(theme.text_secondary)));
        });
        container
    }
}
