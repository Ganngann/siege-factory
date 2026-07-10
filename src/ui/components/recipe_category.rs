use crate::economy::components::{Assembler, BuildingPanel};
use crate::economy::discovery::GlobalArchive;
use crate::economy::recipe::RecipeRegistry;
use crate::economy::resource::ResourceRegistry;
use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::context::UiDataContext;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::theme::Theme;
use crate::ui::registry::ComponentRegistry;
use super::recipe_row::RecipeRow;

pub struct RecipeCategoryComponent;
impl UiComponent for RecipeCategoryComponent {
    fn id(&self) -> &str { "recipe_category" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &UiDataContext, _theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        let category = config.get("category").and_then(|v| v.as_str()).unwrap_or("").to_string();
        spawn_child(commands, parent, (
            Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0), ..default() },
            BackgroundColor(Color::NONE),
            RecipeCategory(category),
        ))
    }
}

#[derive(Component, Clone)]
pub struct RecipeCategory(pub String);

pub fn populate_recipe_categories(
    mut commands: Commands,
    q: Query<(Entity, &RecipeCategory), Added<RecipeCategory>>,
    panel: Res<BuildingPanel>,
    recipes: Res<RecipeRegistry>,
    archive: Res<GlobalArchive>,
    resource_registry: Res<ResourceRegistry>,
    asm_q: Query<&Assembler>,
) {
    let Some(inspected) = panel.inspected else { return; };
    let Ok(asm) = asm_q.get(inspected) else { return; };
    let current_id = &asm.recipe_id;

    for (entity, cat) in &q {
        let mut cat_recipes: Vec<&crate::economy::recipe::RecipeDef> = recipes.recipes
            .values()
            .filter(|r| r.category == cat.0)
            .filter(|r| archive.is_unlocked(&r.id))
            .collect();
        cat_recipes.sort_by(|a, b| a.id.cmp(&b.id));

        commands.entity(entity).with_children(|parent| {
            if cat_recipes.is_empty() { return; }
            parent.spawn((
                Text::new(format!("-- {} --", cat.0.to_uppercase())),
                tf(10.0),
                TextColor(Color::srgb(0.90, 0.80, 0.20)),
            ));

            for recipe in &cat_recipes {
                let is_active = &recipe.id == current_id;
                let input_str: String = recipe.input.iter()
                    .map(|(rid, amt)| format!("{} {}×", resource_registry.display_name(rid), amt))
                    .collect::<Vec<_>>().join(" + ");
                let output_str: String = recipe.output.iter()
                    .map(|(rid, amt)| format!("{} {}×", resource_registry.display_name(rid), amt))
                    .collect::<Vec<_>>().join(" + ");

                let prefix = if is_active { "▸" } else { " " };
                let bg = if is_active { Color::srgba(0.20, 0.50, 0.20, 1.0) } else { Color::srgba(0.12, 0.12, 0.20, 1.0) };
                let text_color = if is_active { Color::srgb(0.40, 0.90, 0.40) } else { Color::srgb(0.60, 0.60, 0.75) };

                parent.spawn((
                    RecipeRow { recipe_id: recipe.id.clone() },
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(28.0),
                        padding: UiRect::horizontal(Val::Px(4.0)),
                        flex_direction: FlexDirection::Column,
                        margin: UiRect::vertical(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(bg),
                )).with_children(|btn| {
                    btn.spawn((Text::new(format!("{}  {} → {}  {:.0}s", prefix, input_str, output_str, recipe.time_sec)), tf(11.0), TextColor(text_color)));
                });
            }
        });
    }
}
