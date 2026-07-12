use bevy::prelude::*;
use crate::economy::recipe::RecipeRegistry;
use crate::player::crafting::CraftingProgress;
use crate::ui::context::UiDataContext;
use crate::ui::registry::{spawn_child, ComponentRegistry, UiComponent};
use crate::ui::theme::Theme;

pub struct HandCraftingProgressComponent;
impl UiComponent for HandCraftingProgressComponent {
    fn id(&self) -> &str { "hand_crafting_progress" }
    fn render(&self, commands: &mut Commands, parent: Entity, _config: &toml::Value, _data: &UiDataContext, _theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        spawn_child(commands, parent, (
            Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0), height: Val::Px(20.0), ..default() },
            BackgroundColor(Color::NONE),
            HandCraftingProgress,
        ))
    }
}

#[derive(Component)]
pub struct HandCraftingProgress;

pub fn update_hand_crafting_progress(
    progress: Res<CraftingProgress>,
    recipes: Res<RecipeRegistry>,
    mut q: Query<&mut Text, With<HandCraftingProgress>>,
) {
    let Some(ref recipe_id) = progress.active_recipe else {
        for mut text in q.iter_mut() {
            text.0 = String::new();
        }
        return;
    };
    let Some(recipe) = recipes.get(recipe_id) else { return };
    let pct = (progress.timer / recipe.time_sec * 100.0).min(100.0) as u32;
    for mut text in q.iter_mut() {
        text.0 = format!("Crafting {}... {}%", recipe_id, pct);
    }
}
