use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct RecipeNameComponent;
impl UiComponent for RecipeNameComponent {
    fn id(&self) -> &str { "recipe_name" }
    fn render(&self, commands: &mut Commands, parent: Entity, _config: &toml::Value, data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, _registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let name = data.resolve("recipe.name");
        spawn_child(commands, parent, (Text::new(name), tf(theme.font_size_body), TextColor(theme.text_primary)))
    }
}
