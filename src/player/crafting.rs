use bevy::prelude::*;

use crate::core::game_font::tf;

use crate::core::utils::silent_despawn;
use crate::economy::components::Player;
use crate::economy::recipe::{RecipeDef, RecipeRegistry};
use crate::economy::resource::Inventory;
use crate::economy::window::{BG_SECTION, spawn_window};

// ── Resources ──

#[derive(Resource, Default)]
pub struct CraftingOpen(pub bool);

#[derive(Resource, Default)]
pub struct CraftingProgress {
    pub active_recipe: Option<String>,
    pub timer: f32,
}

// ── Marker component ──

#[derive(Component)]
pub struct CraftingPanel;

#[derive(Component)]
pub struct CraftButton {
    pub recipe_id: String,
}

#[derive(Component)]
pub struct CraftingProgressText;

// ── Helper ──

pub fn filter_hand_recipes(registry: &RecipeRegistry) -> Vec<&RecipeDef> {
    registry
        .recipes
        .values()
        .filter(|r| r.craftable_in.iter().any(|s| s == "hand"))
        .collect()
}

fn can_afford(inv: &Inventory, recipe: &RecipeDef) -> bool {
    recipe
        .input
        .iter()
        .all(|(rid, amount)| inv.get(rid) >= *amount)
}

// ── Systems ──

pub fn crafting_input(keys: Res<ButtonInput<KeyCode>>, mut open: ResMut<CraftingOpen>) {
    if keys.just_pressed(KeyCode::KeyC) {
        open.0 = !open.0;
    }
}

pub fn spawn_crafting_panel(
    mut commands: Commands,
    open: Res<CraftingOpen>,
    panel_query: Query<Entity, With<CraftingPanel>>,
    recipe_registry: Res<RecipeRegistry>,
    player_query: Query<Entity, With<Player>>,
) {
    if !open.is_changed() {
        return;
    }

    // Close
    if !open.0 {
        for entity in panel_query.iter() {
            silent_despawn(&mut commands, entity);
        }
        return;
    }

    // Already open
    if !panel_query.is_empty() {
        return;
    }

    let Ok(_player_entity) = player_query.single() else {
        return;
    };

    let hand_recipes = filter_hand_recipes(&recipe_registry);
    let recipe_count = hand_recipes.len().max(1);
    let row_height = 52.0;
    let header_height = 36.0;
    let padding = 16.0;
    let w = 320.0;
    let h = padding + header_height + recipe_count as f32 * row_height + padding;

    let panel_root = spawn_window(
        &mut commands,
        "Crafting",
        w,
        h,
        120.0,
        120.0,
        None,
        |parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(4.0),
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|col| {
                    for recipe in &hand_recipes {
                        let rid = recipe.id.clone();
                        let input_text: String = recipe
                            .input
                            .iter()
                            .map(|(r, a)| format!("{} x{}", r.display_name(), a))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let output_text: String = recipe
                            .output
                            .iter()
                            .map(|(r, a)| format!("{} x{}", r.display_name(), a))
                            .collect::<Vec<_>>()
                            .join(", ");

                        col.spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(6.0)),
                                ..default()
                            },
                            BackgroundColor(BG_SECTION),
                        ))
                        .with_children(|row| {
                            row.spawn(Text::new(format!("{} -> {}", input_text, output_text)));
                            row.spawn((
                                CraftButton {
                                    recipe_id: rid.clone(),
                                },
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(28.0),
                                    margin: UiRect::top(Val::Px(4.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.2, 0.4, 0.2)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Craft"),
                                    tf(13.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });
                    }
                });
        },
    );
    commands.entity(panel_root).insert(CraftingPanel);
}

pub fn craft_button_system(
    // SUGGEST: type CraftButtonQuery = Query<(&Interaction, &CraftButton), (Changed<Interaction>, With<Button>)> (clippy::type_complexity)
    mut interaction_query: Query<
        (&Interaction, &CraftButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut player_query: Query<&mut Inventory, With<Player>>,
    recipe_registry: Res<RecipeRegistry>,
    mut progress: ResMut<CraftingProgress>,
) {
    for (interaction, btn) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if progress.active_recipe.is_some() {
            continue;
        }

        let Ok(mut inv) = player_query.single_mut() else {
            continue;
        };

        let Some(recipe) = recipe_registry.get(&btn.recipe_id) else {
            continue;
        };

        if !can_afford(&inv, recipe) {
            continue;
        }

        for (rid, amount) in &recipe.input {
            inv.remove(rid, *amount);
        }

        progress.active_recipe = Some(btn.recipe_id.clone());
        progress.timer = 0.0;
    }
}

pub fn crafting_tick(
    time: Res<Time>,
    recipe_registry: Res<RecipeRegistry>,
    mut progress: ResMut<CraftingProgress>,
    mut player_query: Query<&mut Inventory, With<Player>>,
    mut toast_queue: ResMut<crate::core::toast::ToastQueue>,
) {
    let Some(ref recipe_id) = progress.active_recipe else {
        return;
    };

    let Some(recipe) = recipe_registry.get(recipe_id) else {
        progress.active_recipe = None;
        return;
    };

    progress.timer += time.delta_secs();
    if progress.timer >= recipe.time_sec {
        let Ok(mut inv) = player_query.single_mut() else {
            return;
        };

        for (rid, amount) in &recipe.output {
            inv.add(rid, *amount);
        }

        let name = recipe
            .output
            .iter()
            .map(|(r, a)| format!("{} x{}", r.display_name(), a))
            .collect::<Vec<_>>()
            .join(", ");
        toast_queue.0.push(format!("Crafted {}", name));

        progress.active_recipe = None;
        progress.timer = 0.0;
    }
}

pub fn update_crafting_progress_text(
    progress: Res<CraftingProgress>,
    recipe_registry: Res<RecipeRegistry>,
    mut query: Query<&mut Text, With<CraftingProgressText>>,
) {
    if !progress.is_changed() {
        return;
    }

    for mut text in query.iter_mut() {
        if let Some(ref recipe_id) = progress.active_recipe {
            if let Some(recipe) = recipe_registry.get(recipe_id) {
                let pct = (progress.timer / recipe.time_sec * 100.0).min(100.0) as u32;
                let name = recipe
                    .output
                    .iter()
                    .map(|(r, _)| r.display_name())
                    .collect::<Vec<_>>()
                    .join(", ");
                text.0 = format!("Crafting {}... {}%", name, pct);
            } else {
                text.0 = String::new();
            }
        } else {
            text.0 = String::new();
        }
    }
}

pub fn cleanup_crafting_panel(
    mut commands: Commands,
    panel_query: Query<Entity, With<CraftingPanel>>,
) {
    for entity in panel_query.iter() {
        silent_despawn(&mut commands, entity);
    }
}
