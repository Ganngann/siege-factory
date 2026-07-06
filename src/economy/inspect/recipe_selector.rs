use crate::economy::building::BuildingRegistry;
use crate::economy::components::{
    Assembler, Building, BuildingPanel, CloseButton, DiscoveredRecipes, RecipeCategoryLabel,
    RecipeChangeButton, RecipeSelectorItem, RecipeSelectorRoot,
};
use crate::economy::discovery::GlobalArchive;
use crate::economy::recipe::RecipeRegistry;
use crate::economy::resource::{Inventory, ResourceRegistry};
use crate::economy::window::{BTN_CLOSE, TEXT_GREEN, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_YELLOW};
use bevy::prelude::*;

// ── Recipe change button → open selector ──

pub fn recipe_change_system(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    query: Query<&Interaction, (Changed<Interaction>, With<RecipeChangeButton>)>,
    building_query: Query<&Building>,
    assembler_query: Query<&Assembler>,
    inventory_query: Query<Option<&Inventory>>,
    discovered_query: Query<Option<&DiscoveredRecipes>>,
    recipes: Res<RecipeRegistry>,
    resource_registry: Res<ResourceRegistry>,
    reg: Res<BuildingRegistry>,
    global_archive: Res<GlobalArchive>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(inspected) = panel.inspected else {
            continue;
        };
        let Ok(building) = building_query.get(inspected) else {
            continue;
        };
        let Ok(asm) = assembler_query.get(inspected) else {
            continue;
        };

        if let Some(e) = panel.recipe_selector.take() {
            commands.entity(e).try_despawn();
        }

        let categories = reg
            .get(&building.kind)
            .map(|def| def.recipe_categories.clone())
            .unwrap_or_default();

        let building_inv = inventory_query.get(inspected).ok().and_then(|o| o);

        // Build set of unlocked recipes for this building
        let mut unlocked = global_archive.unlocked_recipes.clone();
        if let Ok(Some(discovered)) = discovered_query.get(inspected) {
            for id in &discovered.0 {
                unlocked.insert(id.clone());
            }
        }
        // Always show the currently selected recipe
        unlocked.insert(asm.recipe_id.clone());

        let sel = spawn_recipe_selector(
            &mut commands,
            &asm.recipe_id,
            &categories,
            &recipes,
            &resource_registry,
            building_inv,
            &unlocked,
        );
        if let Some(root) = panel.root {
            commands.entity(root).add_child(sel);
        }
        panel.recipe_selector = Some(sel);
    }
}

fn spawn_recipe_selector(
    commands: &mut Commands,
    current_id: &str,
    categories: &[String],
    recipes: &RecipeRegistry,
    resource_registry: &ResourceRegistry,
    building_inv: Option<&Inventory>,
    unlocked: &std::collections::HashSet<String>,
) -> Entity {
    commands
        .spawn((
            RecipeSelectorRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(super::RECIPE_SELECTOR_WIDTH),
                top: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                width: Val::Px(360.0),
                height: Val::Px(super::RECIPE_SELECTOR_HEIGHT),
                padding: UiRect::all(Val::Px(10.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.10, 0.20, 0.98)),
            Outline {
                width: Val::Px(1.0),
                offset: Val::ZERO,
                color: Color::srgb(0.40, 0.40, 0.55),
            },
            ZIndex(102),
        ))
        .with_children(|parent| {
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },))
                .with_children(|title| {
                    title.spawn((
                        Text::new("Select Recipe"),
                        TextFont::from_font_size(super::CLOSE_BUTTON_FONT),
                        TextColor(TEXT_PRIMARY),
                    ));
                    title
                        .spawn((
                            CloseButton,
                            Button,
                            Node {
                                width: Val::Px(20.0),
                                height: Val::Px(20.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_CLOSE),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("X"),
                                TextFont::from_font_size(12.0),
                                TextColor(Color::WHITE),
                            ));
                        });
                });

            let mut seen_categories: Vec<(String, Vec<&crate::economy::recipe::RecipeDef>)> =
                Vec::new();
            for cat in categories {
                let mut cat_recipes: Vec<&crate::economy::recipe::RecipeDef> = recipes
                    .recipes
                    .values()
                    .filter(|r| r.category == *cat)
                    .filter(|r| unlocked.contains(&r.id))
                    .collect();
                cat_recipes.sort_by(|a, b| a.id.cmp(&b.id));
                if !cat_recipes.is_empty() {
                    seen_categories.push((cat.clone(), cat_recipes));
                }
            }

            for (cat_name, cat_recipes) in &seen_categories {
                parent.spawn((
                    RecipeCategoryLabel,
                    Text::new(format!("-- {} --", cat_name.to_uppercase())),
                    TextFont::from_font_size(super::SECTION_FONT_SIZE),
                    TextColor(TEXT_YELLOW),
                    Node {
                        margin: UiRect::vertical(Val::Px(4.0)),
                        ..default()
                    },
                ));

                for recipe in cat_recipes {
                    let is_active = recipe.id == current_id;

                    let can_craft = if recipe.input.is_empty() {
                        recipe.input.is_empty()
                    } else {
                        building_inv.map_or(false, |inv| {
                            recipe.input.iter().all(|(rid, amt)| inv.get(rid) >= *amt)
                        })
                    };

                    let bg = if is_active {
                        Color::srgb(0.20, 0.50, 0.20)
                    } else if can_craft {
                        Color::srgb(0.18, 0.35, 0.18)
                    } else {
                        Color::srgb(0.12, 0.12, 0.20)
                    };
                    let border_color = if can_craft && !is_active {
                        Color::srgb(0.30, 0.70, 0.30)
                    } else {
                        Color::srgb(0.20, 0.20, 0.30)
                    };
                    let prefix = if is_active {
                        "> "
                    } else if can_craft {
                        "[x] "
                    } else {
                        "    "
                    };

                    let input_str: String = recipe
                        .input
                        .iter()
                        .map(|(rid, amt)| {
                            let name = resource_registry.display_name(rid);
                            format!("{} x{}", name, amt)
                        })
                        .collect::<Vec<_>>()
                        .join(" + ");
                    let output_str: String = recipe
                        .output
                        .iter()
                        .map(|(rid, amt)| {
                            let name = resource_registry.display_name(rid);
                            format!("{} x{}", name, amt)
                        })
                        .collect::<Vec<_>>()
                        .join(" + ");

                    parent
                        .spawn((
                            RecipeSelectorItem {
                                recipe_id: recipe.id.clone(),
                            },
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(34.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(6.0)),
                                margin: UiRect::vertical(Val::Px(1.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(bg),
                            BorderColor::all(border_color),
                        ))
                        .with_children(|btn| {
                            let recipe_name = resource_registry
                                .get_opt(&recipe.id)
                                .map_or(recipe.id.as_str(), |r| &r.name);
                            btn.spawn((
                                Text::new(format!("{}{}", prefix, recipe_name)),
                                TextFont::from_font_size(12.0),
                                TextColor(if is_active {
                                    TEXT_GREEN
                                } else if can_craft {
                                    TEXT_PRIMARY
                                } else {
                                    TEXT_SECONDARY
                                }),
                            ));
                            btn.spawn((
                                Text::new(format!(
                                    "    {}  ->  {}  |  {:.1}s",
                                    input_str, output_str, recipe.time_sec
                                )),
                                TextFont::from_font_size(10.0),
                                TextColor(TEXT_SECONDARY),
                            ));
                        });
                }
            }
        })
        .id()
}

// ── Recipe selector item click ──

pub fn recipe_selector_click(
    mut commands: Commands,
    mut panel: ResMut<BuildingPanel>,
    query: Query<(&Interaction, &RecipeSelectorItem), Changed<Interaction>>,
    mut assembler_query: Query<&mut Assembler>,
) {
    for (interaction, item) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(inspected) = panel.inspected else {
            continue;
        };
        if let Ok(mut asm) = assembler_query.get_mut(inspected) {
            asm.recipe_id = item.recipe_id.clone();
            panel.dirty = true;
        }
        if let Some(e) = panel.recipe_selector.take() {
            commands.entity(e).try_despawn();
        }
        return;
    }
}
