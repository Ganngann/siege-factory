use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::economy::discovery::GlobalArchive;
use crate::economy::recipe::RecipeRegistry;
use crate::economy::resource::ResourceRegistry;
use crate::player::crafting::{CraftButton, CraftingProgress};
use crate::ui::context::UiDataContext;
use crate::ui::registry::{spawn_child, ComponentRegistry, UiComponent};
use crate::ui::theme::Theme;

pub struct HandCraftingListComponent;
impl UiComponent for HandCraftingListComponent {
    fn id(&self) -> &str { "hand_crafting_list" }
    fn render(&self, commands: &mut Commands, parent: Entity, _config: &toml::Value, _data: &UiDataContext, _theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        spawn_child(commands, parent, (
            Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0), ..default() },
            BackgroundColor(Color::NONE),
            HandCraftingList,
        ))
    }
}

#[derive(Component)]
pub struct HandCraftingList;

pub fn populate_hand_crafting_list(
    mut commands: Commands,
    q: Query<Entity, Added<HandCraftingList>>,
    recipes: Res<RecipeRegistry>,
    archive: Res<GlobalArchive>,
    resource_registry: Res<ResourceRegistry>,
    progress: Res<CraftingProgress>,
) {
    let active_id = progress.active_recipe.as_deref();

    let mut hand_recipes: Vec<_> = recipes
        .recipes
        .values()
        .filter(|r| r.craftable_in.iter().any(|s| s == "hand"))
        .filter(|r| archive.is_unlocked(&r.id))
        .collect();
    hand_recipes.sort_by(|a, b| a.id.cmp(&b.id));

    for entity in &q {
        commands.entity(entity).with_children(|parent| {
            if hand_recipes.is_empty() {
                parent.spawn((
                    Text::new("No hand recipes available"),
                    tf(11.0),
                    TextColor(Color::srgb(0.60, 0.60, 0.75)),
                ));
                return;
            }

            for recipe in &hand_recipes {
                let is_active = active_id == Some(recipe.id.as_str());
                let input_str: String = recipe
                    .input
                    .iter()
                    .map(|(rid, amt)| format!("{} {}×", resource_registry.display_name(rid), amt))
                    .collect::<Vec<_>>().join(" + ");
                let output_str: String = recipe
                    .output
                    .iter()
                    .map(|(rid, amt)| format!("{} {}×", resource_registry.display_name(rid), amt))
                    .collect::<Vec<_>>().join(" + ");

                let bg = if is_active { Color::srgba(0.20, 0.50, 0.20, 1.0) } else { Color::srgba(0.12, 0.12, 0.20, 1.0) };
                let text_color = if is_active { Color::srgb(0.40, 0.90, 0.40) } else { Color::srgb(0.80, 0.80, 0.90) };

                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(6.0)),
                        margin: UiRect::vertical(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(bg),
                )).with_children(|row| {
                    row.spawn((Text::new(format!("{} → {}", input_str, output_str)), tf(11.0), TextColor(text_color)));
                    if !is_active {
                        row.spawn((
                            CraftButton { recipe_id: recipe.id.clone() }, Button,
                            Node { width: Val::Percent(100.0), height: Val::Px(24.0), margin: UiRect::top(Val::Px(4.0)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                            BackgroundColor(Color::srgb(0.2, 0.4, 0.2)),
                        )).with_children(|btn| {
                            btn.spawn((Text::new("Craft"), tf(12.0), TextColor(Color::WHITE)));
                        });
                    }
                });
            }
        });
    }
}
