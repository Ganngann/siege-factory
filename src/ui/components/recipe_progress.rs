use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct RecipeProgressComponent;
impl UiComponent for RecipeProgressComponent {
    fn id(&self) -> &str { "recipe_progress" }
    fn render(&self, commands: &mut Commands, parent: Entity, _config: &toml::Value, data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, _registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let pct: f32 = data.resolve("recipe.progress").parse().unwrap_or(0.0);
        let time_sec: f32 = data.resolve("recipe.time_sec").parse().unwrap_or(0.0);
        let container = spawn_child(commands, parent, (Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0), ..default() }, BackgroundColor(Color::NONE)));
        commands.entity(container).with_children(|p| {
            p.spawn((Node { width: Val::Percent(100.0), height: Val::Px(14.0), ..default() }, BackgroundColor(theme.bar_bg))).with_children(|bg| {
                bg.spawn((Node { width: Val::Percent(pct * 100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(theme.progress_fill)));
            });
            p.spawn((Text::new(format!("{:.1}s / {:.1}s", pct * time_sec, time_sec)), tf(theme.font_size_small), TextColor(theme.text_secondary)));
        });
        container
    }
}
